//! The bidirectional JSON-RPC connection both roles are built on.

use crate::{
    codec::{self, Message},
    handler::Handler,
};
use serde::{Deserialize, Serialize, de::DeserializeOwned};
use std::{
    collections::HashMap,
    sync::{
        Arc, Mutex,
        atomic::{AtomicI64, Ordering},
    },
};
use tokio::{
    io::{AsyncBufRead, AsyncWrite, BufReader},
    sync::{mpsc, oneshot},
    task::AbortHandle,
};

type Pending =
    Arc<Mutex<HashMap<proto::RequestId, oneshot::Sender<Result<serde_json::Value, proto::Error>>>>>;
type InFlight = Arc<Mutex<HashMap<proto::RequestId, AbortHandle>>>;

/// A handle for calling the other side. Cheap to clone.
#[derive(Clone)]
pub struct Peer(Arc<Inner>);

struct Inner {
    outgoing: mpsc::UnboundedSender<Message>,
    pending: Pending,
    next_id: AtomicI64,
}

impl Peer {
    /// Wire a reader and a writer to `handler`, spawning the read and write
    /// loops. The returned [`Peer`] calls the other side.
    pub fn new<R, W, H>(reader: R, writer: W, handler: Arc<H>) -> Self
    where
        R: AsyncBufRead + Unpin + Send + 'static,
        W: AsyncWrite + Unpin + Send + 'static,
        H: Handler,
    {
        let (outgoing, rx) = mpsc::unbounded_channel();
        let peer = Self(Arc::new(Inner {
            outgoing,
            pending: Pending::default(),
            next_id: AtomicI64::new(0),
        }));
        tokio::spawn(write_loop(writer, rx));
        tokio::spawn(read_loop(reader, handler, peer.clone()));
        peer
    }

    /// Wire up a duplex stream — a subprocess's stdio, a socket, an in-memory pipe.
    pub fn duplex<S, H>(stream: S, handler: Arc<H>) -> Self
    where
        S: tokio::io::AsyncRead + AsyncWrite + Send + 'static,
        H: Handler,
    {
        let (reader, writer) = tokio::io::split(stream);
        Self::new(BufReader::new(reader), writer, handler)
    }

    /// Call the other side and wait for its answer.
    pub async fn request<P: Serialize, R: DeserializeOwned>(
        &self,
        method: &str,
        params: P,
    ) -> Result<R, proto::Error> {
        let id = proto::RequestId::Num(self.0.next_id.fetch_add(1, Ordering::Relaxed));
        let params = serde_json::to_value(params)?;

        let (tx, rx) = oneshot::channel();
        self.0
            .pending
            .lock()
            .expect("pending map poisoned")
            .insert(id.clone(), tx);

        if self
            .0
            .outgoing
            .send(Message::request(id.clone(), method, params))
            .is_err()
        {
            self.0
                .pending
                .lock()
                .expect("pending map poisoned")
                .remove(&id);
            return Err(disconnected());
        }

        let value = rx.await.map_err(|_| disconnected())??;
        Ok(serde_json::from_value(value)?)
    }

    /// Tell the other side something. Returns once queued for writing.
    pub fn notify<P: Serialize>(&self, method: &str, params: P) -> Result<(), proto::Error> {
        let params = serde_json::to_value(params)?;
        self.0
            .outgoing
            .send(Message::notification(method, params))
            .map_err(|_| disconnected())
    }

    fn resolve(&self, id: &proto::RequestId, outcome: Result<serde_json::Value, proto::Error>) {
        let waiter = self
            .0
            .pending
            .lock()
            .expect("pending map poisoned")
            .remove(id);
        if let Some(waiter) = waiter {
            let _ = waiter.send(outcome);
        }
    }

    /// Fail every outstanding call, so callers wake instead of hanging.
    fn fail_all(&self) {
        let pending = std::mem::take(&mut *self.0.pending.lock().expect("pending map poisoned"));
        for (_, waiter) in pending {
            let _ = waiter.send(Err(disconnected()));
        }
    }

    fn send(&self, message: Message) {
        let _ = self.0.outgoing.send(message);
    }
}

fn disconnected() -> proto::Error {
    proto::Error::internal_error().data("connection closed")
}

async fn write_loop<W: AsyncWrite + Unpin>(
    mut writer: W,
    mut rx: mpsc::UnboundedReceiver<Message>,
) {
    while let Some(message) = rx.recv().await {
        if codec::write(&mut writer, &message).await.is_err() {
            break;
        }
    }
}

async fn read_loop<R, H>(mut reader: R, handler: Arc<H>, peer: Peer)
where
    R: AsyncBufRead + Unpin,
    H: Handler,
{
    let in_flight = InFlight::default();
    loop {
        let message = match codec::read(&mut reader).await {
            Ok(Some(message)) => message,
            Ok(None) => break,
            // A malformed frame is the peer's bug, not ours; report and continue.
            Err(e) => {
                peer.send(Message::error(None, e));
                continue;
            }
        };
        match classify(message) {
            Frame::Response { id, outcome } => peer.resolve(&id, outcome),
            Frame::Notification { method, params } => {
                if method == proto::method::CANCEL_REQUEST {
                    cancel(&in_flight, &params);
                } else {
                    handler.notification(method, params).await;
                }
            }
            // Spawned, not awaited: a handler that blocks on a user prompt
            // must not stall the notification stream behind it. Responses
            // carry their id, so finishing out of order is correct.
            Frame::Request { id, method, params } => {
                let handler = Arc::clone(&handler);
                let peer = peer.clone();
                let done = Arc::clone(&in_flight);
                let task = tokio::spawn({
                    let id = id.clone();
                    async move {
                        let message = match handler.request(method, params).await {
                            Ok(result) => Message::response(id.clone(), result),
                            Err(e) => Message::error(Some(id.clone()), e),
                        };
                        done.lock().expect("in-flight map poisoned").remove(&id);
                        peer.send(message);
                    }
                });
                in_flight
                    .lock()
                    .expect("in-flight map poisoned")
                    .insert(id, task.abort_handle());
            }
            Frame::Invalid => peer.send(Message::error(None, proto::Error::invalid_request())),
        }
    }
    peer.fail_all();
}

fn cancel(in_flight: &InFlight, params: &serde_json::Value) {
    #[derive(Deserialize)]
    #[serde(rename_all = "camelCase")]
    struct Cancel {
        request_id: proto::RequestId,
    }

    let Ok(Cancel { request_id }) = serde_json::from_value(params.clone()) else {
        return;
    };
    let task = in_flight
        .lock()
        .expect("in-flight map poisoned")
        .remove(&request_id);
    if let Some(task) = task {
        task.abort();
    }
}

enum Frame {
    Request {
        id: proto::RequestId,
        method: String,
        params: serde_json::Value,
    },
    Notification {
        method: String,
        params: serde_json::Value,
    },
    Response {
        id: proto::RequestId,
        outcome: Result<serde_json::Value, proto::Error>,
    },
    Invalid,
}

fn classify(message: Message) -> Frame {
    let Message {
        id,
        method,
        params,
        result,
        error,
        ..
    } = message;
    let params = params.unwrap_or(serde_json::Value::Null);
    match (id, method) {
        (Some(id), Some(method)) => Frame::Request { id, method, params },
        (None, Some(method)) => Frame::Notification { method, params },
        (Some(id), None) => {
            let outcome = match error {
                Some(e) => Err(e),
                // An omitted `result` on a successful response means "no
                // payload"; every empty ACP response deserializes from `{}`.
                None => Ok(result.unwrap_or_else(|| serde_json::json!({}))),
            };
            Frame::Response { id, outcome }
        }
        (None, None) => Frame::Invalid,
    }
}
