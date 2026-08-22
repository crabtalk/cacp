# An event loop instead of a trait

A frontend has a loop already, so `cacp-events` hands it one rather than a trait
to implement. Every call from the agent arrives as an `Event` on a channel.

```rust,ignore
let (client, mut events) = cacp_events::channel();
let (agent, _child) = cacp::spawn(&mut Command::new("my-agent"), client)?;

while let Some(event) = events.recv().await {
    match event {
        Event::Update(notification) => draw(notification.update),
        Event::Permission(request, reply) => reply.send(ask_the_user(request).await),
        _ => {}
    }
}
```

A variant carrying a `Reply` is a request: answer it, or drop the reply to
decline. Dropping sends `method not found` — exactly what the `Client` trait
sends for a method you never implemented — so the `_ => {}` above serves nothing
but updates and permission, and says so on the wire.

That transparency is the whole design. A consumer that ignores an `Event` is
indistinguishable from one that never implemented that method, which is why the
adapter needs no builder and no opt-in list.

## What it inherits

Notifications reach the channel in wire order. A request is dispatched on its
own task, so it can arrive just after a notification that followed it on the
wire.

A `Reply` also knows when the agent gave up: `is_cancelled()` and
`cancelled().await` fire when `$/cancel_request` aborts the request, which is
the cue to take a permission prompt back off the screen.

## When to implement `Client` directly instead

The channel is not always the shorter path. Answer permission by policy rather
than by asking a human — auto-approve, or refuse everything — and routing it out
to a loop and back is strictly more code than a two-method `impl Client`.
