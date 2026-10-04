---
title: "Contributing to Garage: Two Pull Requests, Two Ways In"
published_at: 2026-06-29
carved: false
kind: post
description: "I bet my platform on Garage, the open-source S3 engine. So I started fixing it. Two merged pull requests - a one-line CORS fix I needed, and a bulk-delete compatibility fix I went looking for - and what they taught me about owning your tools."
quote: Sovereignty includes being able to fix the stack you depend on.
tags:
  - garage
  - open-source
  - rust
  - object-storage
  - contributing
---

I'm building a platform on top of Garage. Storm Buckets - the S3-compatible object storage - runs on it. A project I don't maintain sits directly underneath the product I do, and its bugs are, in a real way, my bugs too.

What you can do about that depends on whether you own your tools or rent them. With most of the infrastructure I've paid for over the years, a bug was something I reported and then waited on. Garage is open source, so when I hit one, the source is right there in front of me and I can go in and fix it myself. That's what I ended up doing twice this spring - two patches, both merged upstream now. They came at me from opposite directions: one I needed badly, and one I had no real reason to touch. Writing them taught me more about contributing to open source than any amount of reading about it had.

## The one I needed

The first came out of building the Storm file browser, a small Svelte app that lets a customer click through the objects in their bucket from the browser, without ever opening a terminal or learning what an S3 client is. It turned out to be a good way to run into the parts of S3 that only break in a browser.

Browsers are cautious about talking to other origins. Before mine would send a signed request to my Garage endpoint, it sent a preflight - a small `OPTIONS` request that asks the server, ahead of time, whether the real request will be allowed, with the headers it's planning to send. A signed S3 call carries an `Authorization` header and a handful of `x-amz-*` headers, so the preflight asks about exactly those. If the server's answer doesn't cover them, the browser quietly gives up and the real request never goes out.

Mine were never going out. I spent a while assuming I'd misconfigured something on my own end before I read the preflight response and noticed that it listed the methods it allowed and said nothing at all about headers. The bucket was being addressed by a local name, and on that path Garage's reply simply didn't include the header that would let my signed request through.

So I went and read that corner of Garage. It lives in `src/api/common/cors.rs`, in the permissive `OPTIONS` placeholder, and it was setting `Access-Control-Allow-Methods` without ever setting `Access-Control-Allow-Headers` right beside it. The fix was the one line it was missing:

```rust
// src/api/common/cors.rs, the OPTIONS placeholder
.header(ACCESS_CONTROL_ALLOW_METHODS, "*")
.header(ACCESS_CONTROL_ALLOW_HEADERS, "*")  // <- the line I added
```

That was the whole change, and it's about as small as a contribution gets ([PR #1450](https://git.deuxfleurs.fr/Deuxfleurs/garage/pulls/1450)). I think that's a perfectly good way to start. I wasn't trying to redesign anything; I had a problem I could follow down into the source, and by the time I'd followed it I understood a little more of the engine I depend on than I had that morning. The patch was trivial to write, and the work was all in the reading that led to it.

## The one I went looking for

The second one had nothing to do with me. I was reading through Garage's open issues one evening - partly diligence, partly just wanting to know the failure modes of the thing I'm betting on - when I came across [#1460](https://git.deuxfleurs.fr/Deuxfleurs/garage/issues/1460). Bulk `DeleteObjects` was returning errors for keys that didn't exist, and I figured I could probably fix it.

The detail underneath it is more interesting than it first sounds. S3 lets you delete a batch of keys in a single request, and the open question is what should happen when one of the keys in your batch isn't actually there. AWS settled this a long time ago: deleting something that's already gone counts as a success, and it comes back in the `<Deleted>` part of the response rather than the `<Error>` part. Garage was treating that same case as a per-object error.

On its own, that's a defensible reading. But S3-compatibility is really a promise that tools written for Amazon will work against you without changes, and some of those tools treat any per-item error in a batch delete as fatal - Elasticsearch's snapshot repository was the one biting people in the issue. They'd run fine against AWS and then fall over against Garage on an operation that was supposed to be a no-op.

Almost all of the work here was understanding that. I read the AWS documentation for `DeleteObjects` closely, worked out what Elasticsearch expected to get back, and only once I was sure I had the behavior right did I touch any code. The change itself is small. In `src/api/s3/delete.rs`, a missing key stops being an error and becomes a recorded deletion:

```rust
Err(Error::NoSuchKey) => {
    if cmd.quiet { continue; }
    ret_deleted.push(s3_xml::Deleted {
        key: s3_xml::Value(obj.key.clone()),
        version_id: None,
        delete_marker_version_id: None,
    });
}
```

And in `src/api/s3/xml.rs`, the version fields became optional, so a key that never existed could be reported back without inventing version information it never had:

```rust
#[serde(rename = "VersionId", skip_serializing_if = "Option::is_none")]
pub version_id: Option<Value>,
```

Plus a test that deletes a key which was never there and checks that it comes back marked as deleted. Around thirty-nine lines all told ([PR #1469](https://git.deuxfleurs.fr/Deuxfleurs/garage/pulls/1469), closing #1460), and most of them were only obvious after the reading I'd done first.

## Why I give it back

None of this pays. The afternoon spent reading AWS docs, the evening tracing Garage's source - none of it shows up on an invoice. So why does a one-person company give that time away for free?

Sustainability. One of the three things Storm Developments won't compromise on, next to sovereignty and security.

My services wouldn't exist without open source. My storage runs on Garage. My code lives on Forgejo. This site is served by Caddy. All of it was built and given away by people who didn't have to, and I'm running a company on top of their work.

So I give back. I want Storm Developments to still be here in ten years, and that only holds if the foundations under it stay healthy - stronger S3 compatibility, the bugs I can reach and fix, improvements pushed upstream where every other team running the same tools gets them too. That's a win for me, and a win for the mission that brought me here in the first place: Canadian digital sovereignty.

Both patches are part of Garage now. Whoever spins up a node tonight gets the missing CORS header and the corrected delete behaviour for free - the same way I got the rest of Garage for free. This week that meant two fixes for the engine behind Storm Buckets. It won't be the last.

{~ card ~}
Storm Buckets is S3-compatible object storage hosted entirely in Canada, built on the Garage engine I've been digging into here. It's in open alpha: **[stormdevelopments.ca/buckets](https://stormdevelopments.ca/buckets/)**.
{~~}
