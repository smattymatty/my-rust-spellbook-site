---
title: "Pingora Is My New Best Friend"
published_at: 2026-07-20
carved: false
kind: post
featured: false
description: "Pingora is Cloudflare's Rust framework for building proxies, and it's a framework, not a library: it runs the proxy and calls your code at fixed points. Here's how to build a pass-through proxy in about thirty lines, then teach it to hold and deny a request."
quote: "You don't call Pingora to proxy a request. It proxies the request and calls you."
tags:
  - pingora
  - rust
  - proxies
  - tutorial
  - storm-buckets
  - canadian-digital-sovereignty
---

I needed a program that could sit in the middle of a live upload, pause it halfway, decide something, and then either let the bytes continue or kill the request dead. 

Pingora is what I reached for, and I want to walk you through it, because the thing that took me longest to *get* is the thing that makes it click, and once it clicks you can build a working proxy in about thirty lines.

This is a hands-on tutorial. Bring a terminal.

## What Pingora is

Pingora is Cloudflare's open-source framework for building network proxies, written in Rust. They run a large share of the internet's traffic through it, then open-sourced it. Reverse proxies, load balancers, API gateways, anything that sits between a client and a server and does something to the traffic: this is the toolkit for building your own, instead of configuring someone else's.

**Pingora is a framework, not a library.** You don't call Pingora to proxy a request. Pingora runs the proxy loop and calls *you*, at fixed moments in each request's life. Those moments are called phases. You write only the phases you care about; every phase you leave alone defaults to transparent pass-through. That inversion is why a real proxy is so small: the framework already knows how to accept a connection, stream a body, and reuse an upstream. You just fill in the decisions.

## The phases, in the order they fire

A request travels down through these on the way up to your upstream, and the response travels back up through them on the way down. The ones you'll reach for first:

- **`request_filter`** runs first, before any upstream work. Look at the request, and reject it cheap if you want to. This is where a "deny this outright" decision belongs.
- **`upstream_peer`** is the one phase you *must* implement. It answers the only question with no sensible default: which upstream server do I connect to?
- **`request_body_filter`** runs once per chunk of the request body as it streams upward. Not once per request, once per chunk. This is the seam where you can hold an upload.
- **`response_body_filter`** runs once per chunk of the response streaming back down. Return a duration here and Pingora will delay that chunk, which is how you shape bandwidth.
- **`logging`** always runs at the end, even on errors. Good place to count what happened.

Now build one.

## Your first proxy: thirty lines that forward bytes

Start a project and pull in Pingora:

```bash
cargo new pass-through
cd pass-through
cargo add pingora --features lb
cargo add async-trait
```

The first build pulls a lot and takes a few minutes. While it compiles, here's the whole program. Two imports:

```rust
use async_trait::async_trait;
use pingora::prelude::*;
```

The `#[async_trait]` macro is there because Rust traits can't yet have async methods on their own, and Pingora's phases are async. You'll put it on the impl block.

Now the proxy itself. It's a struct that carries no state today:

```rust
pub struct PassThrough;
```

And the implementation. Two things: a per-request context type, which is unit for now because we keep no state, and the one required phase, `upstream_peer`, which points every request at a server on `127.0.0.1:3900`:

```rust
#[async_trait]
impl ProxyHttp for PassThrough {
    type CTX = ();
    fn new_ctx(&self) -> Self::CTX {}

    async fn upstream_peer(
        &self,
        _session: &mut Session,
        _ctx: &mut Self::CTX,
    ) -> Result<Box<HttpPeer>> {
        // Connect to the upstream: address, TLS off, empty SNI.
        Ok(Box::new(HttpPeer::new(("127.0.0.1", 3900), false, String::new())))
    }
}
```

Then `main`: build a server, bootstrap it, build a proxy service from your struct, bind it to a port, hand it to the server, and run:

```rust
fn main() {
    let mut server = Server::new(None).unwrap();
    server.bootstrap();

    let mut svc = http_proxy_service(&server.configuration, PassThrough);
    svc.add_tcp("127.0.0.1:6188");

    server.add_service(svc);
    server.run_forever();
}
```

That's a complete proxy. It listens on `6188`, forwards every request to whatever's on `3900`, and streams the response back untouched.

To try it, put any HTTP server on 3900 in another terminal:

```bash
python3 -m http.server 3900
```

Then run your proxy and hit it:

```bash
cargo run
curl -v http://127.0.0.1:6188/
```

The response comes back through your proxy from the Python server behind it. Stop the Python server and curl again: you get a connection error from Pingora, because the upstream it tried to reach in `upstream_peer` is gone. You wrote one method and got a working, streaming, error-handling reverse proxy.

{~ alert type="info" ~}
`upstream_peer` is the only phase with no default, because there's no sane guess for "where should this go." Every other phase Pingora can handle transparently until you say otherwise.
{~~}

## Making it observe

A proxy that says nothing is hard to trust. Give it a per-request context to count into, and a couple of phases that narrate. First, the context becomes a real struct instead of unit:

```rust
pub struct Obs { chunks: usize, bytes: usize }
```

```rust
type CTX = Obs;
fn new_ctx(&self) -> Self::CTX { Obs { chunks: 0, bytes: 0 } }
```

Pingora creates one `Obs` per request and threads it, as `&mut`, through every phase of that request. No globals, no locks: one owner per request. Now log the request as it arrives, count each body chunk, and print a summary at the end:

```rust
async fn request_filter(&self, session: &mut Session, _ctx: &mut Self::CTX)
    -> Result<bool> {
    let h = session.req_header();
    eprintln!(">> {} {}", h.method, h.uri);
    Ok(false)   // false = keep going. (true would mean "I answered, stop.")
}

async fn request_body_filter(&self, _session: &mut Session,
    body: &mut Option<Bytes>, end_of_stream: bool, ctx: &mut Self::CTX)
    -> Result<()> {
    if let Some(b) = body { ctx.chunks += 1; ctx.bytes += b.len(); }
    eprintln!("   chunk {}  {}B total  eos={}", ctx.chunks, ctx.bytes, end_of_stream);
    Ok(())
}

async fn logging(&self, session: &mut Session, _e: Option<&pingora::Error>,
    ctx: &mut Self::CTX) {
    let status = session.response_written().map_or(0, |r| r.status.as_u16());
    eprintln!("<< status={}  chunks={}  bytes={}", status, ctx.chunks, ctx.bytes);
}
```

You'll need `use bytes::Bytes;` up top (`cargo add bytes`). Run an upload through it now and watch the phases fire in order: the request line, then a chunk line per piece of body streaming up, then the summary. You're seeing Pingora's request lifecycle on your own screen.

## The three verbs

Now the proxy stops being a pass-through and becomes a gate. There are three things you'll want to do to a request, and Pingora spells two of them right in the trait.

**Deny.** Return an error from `request_filter` and the request aborts before it ever reaches an upstream:

```rust
if should_reject {
    return pingora::Error::e_explain(
        pingora::ErrorType::HTTPStatus(403), "denied by the guard");
}
```

**Delay.** `response_body_filter` returns `Result<Option<Duration>>`. Return `Some(duration)` and Pingora holds that response chunk that long before sending it. That's a bandwidth shaper in one return value.

**Hold.** This is the one that isn't in the docs. I wanted to pause a request body mid-stream: let some chunks arrive, do slow work, and only then let them continue upstream. The documentation never mentions it. So I read Pingora's own source, and found that the forwarding loop *awaits* `request_body_filter` before it writes the chunk upstream. Which means "hold" needs no special API at all. You just do async work inside the phase and await it:

```rust
async fn request_body_filter(&self, _session: &mut Session,
    body: &mut Option<Bytes>, end_of_stream: bool, ctx: &mut Self::CTX)
    -> Result<()> {
    if let Some(b) = body { ctx.chunks += 1; ctx.bytes += b.len(); }

    // Do slow work here, await it, and the chunk cannot forward until you
    // return. Real code does durable I/O; a sleep shows the effect:
    if ctx.chunks == 1 {
        tokio::time::sleep(std::time::Duration::from_secs(2)).await;
    }
    Ok(())
}
```

Upload through that and your transfer takes two seconds longer, then completes normally. The chunk could not leave until the `.await` returned. It's why I could build a guard that reserves space before it forwards a byte: the reservation is the slow work, the `.await` is the hold, and a refused reservation is just a returned error.

{~ alert type="warning" ~}
`request_body_filter` runs per chunk, and the final call can arrive with `end_of_stream = true` and an empty body. Count with a guard (`if let Some(b) = body`) so the empty goodbye doesn't trip you.
{~~}

## Where this goes

A pass-through in thirty lines, then observation, then the three verbs. From here you're picking upstreams dynamically, verifying signatures in `request_filter` before you trust anything downstream, shaping responses in `response_body_filter`. All of it is the same shape you already have: override a phase, return a decision, let the framework run the loop.

I'm using it to build a guard that sits in front of my storage and enforces a hard limit before any byte lands, and I'll write that up once it's carrying real traffic. But you don't need my use case to start. You need a server on `3900` and thirty lines.

The reason I reach for Pingora, past the ergonomics: it's open, it's Rust, it runs on my own hardware, and when the docs went quiet on the one thing I needed, I could read the forwarding loop and find the answer sitting in it. A framework whose behavior I can own beats a proxy service I rent from someone who can change the deal. Same reason I self-host everything else.

Primary source worth your time: Pingora's own [quick start and the phase documentation](https://github.com/cloudflare/pingora). Read the trait. When something isn't in the guide, the source is right there, and that's where the good parts are hiding.
