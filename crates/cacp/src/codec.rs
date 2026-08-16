//! The JSON-RPC 2.0 envelope and its newline-delimited framing.

use serde::{Deserialize, Serialize};
use tokio::io::{AsyncBufRead, AsyncBufReadExt, AsyncWrite, AsyncWriteExt};

/// One frame off the wire.
///
/// Requests, notifications and responses are distinguished by which fields are
/// present, not by a tag — so this is one struct with optional fields rather
/// than an untagged enum, which would silently pick the wrong variant when a
/// peer omits something.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct Message {
    pub jsonrpc: Version,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub id: Option<proto::RequestId>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub method: Option<String>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub params: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub result: Option<serde_json::Value>,
    #[serde(default, skip_serializing_if = "Option::is_none")]
    pub error: Option<proto::Error>,
}

impl Message {
    pub fn request(
        id: proto::RequestId,
        method: impl Into<String>,
        params: serde_json::Value,
    ) -> Self {
        Self {
            jsonrpc: Version,
            id: Some(id),
            method: Some(method.into()),
            params: Some(params),
            result: None,
            error: None,
        }
    }

    pub fn notification(method: impl Into<String>, params: serde_json::Value) -> Self {
        Self {
            jsonrpc: Version,
            id: None,
            method: Some(method.into()),
            params: Some(params),
            result: None,
            error: None,
        }
    }

    pub fn response(id: proto::RequestId, result: serde_json::Value) -> Self {
        Self {
            jsonrpc: Version,
            id: Some(id),
            method: None,
            params: None,
            result: Some(result),
            error: None,
        }
    }

    pub fn error(id: Option<proto::RequestId>, error: proto::Error) -> Self {
        Self {
            jsonrpc: Version,
            id,
            method: None,
            params: None,
            result: None,
            error: Some(error),
        }
    }
}

/// The literal `"2.0"`, checked on the way in and emitted on the way out.
#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Version;

impl Serialize for Version {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str("2.0")
    }
}

impl<'de> Deserialize<'de> for Version {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let raw = String::deserialize(deserializer)?;
        if raw == "2.0" {
            Ok(Self)
        } else {
            Err(serde::de::Error::custom(format!(
                "unsupported jsonrpc version {raw:?}"
            )))
        }
    }
}

/// Read one frame. `Ok(None)` means the peer closed the connection.
pub async fn read<R: AsyncBufRead + Unpin>(
    reader: &mut R,
) -> Result<Option<Message>, proto::Error> {
    let mut line = String::new();
    loop {
        line.clear();
        if reader.read_line(&mut line).await? == 0 {
            return Ok(None);
        }
        // Agents launched through a shell wrapper sometimes emit blank lines.
        if !line.trim().is_empty() {
            return Ok(Some(serde_json::from_str(&line)?));
        }
    }
}

pub async fn write<W: AsyncWrite + Unpin>(
    writer: &mut W,
    message: &Message,
) -> Result<(), proto::Error> {
    let mut buf = serde_json::to_vec(message)?;
    buf.push(b'\n');
    writer.write_all(&buf).await?;
    writer.flush().await?;
    Ok(())
}
