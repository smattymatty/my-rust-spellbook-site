---
title: "us-east-1: The Bane of Sovereignty"
published_at: 2026-09-17
kind: post
featured: false
description: "MSP360 Backup signs every request to my Canadian storage with us-east-1, and there's no field to change it. A hardcoded default can shut independent infrastructure out of a supposedly shared interface without anyone intending it."
quote: The flag should be your choice, including a name with no national flag attached. Ours is storm.
tags:
  - canadian-digital-sovereignty
  - storm-buckets
  - object-storage
  - garage
  - interoperability
  - self-hosting
---

You should get to choose where your data lives, and the flags that get planted on it.

One of our customers ran into this with MSP360 Backup. I installed its Windows client in August 2026 to investigate. Our storage runs Garage with the signing region set to `storm`, and every request I examined was signed with `us-east-1`. I couldn't find a setting to change it, and an endpoint hostname containing `storm` didn't change it either. A forum thread showed someone else had already hit the same wall with Garage. I've asked MSP360 whether there's a setting I missed and offered to test a fix.

For my company, Storm Developments, that choice is Canada. Yours might be somewhere else, in your own country, through a local cooperative, or on a machine you own. Nobody should need to adopt somebody else's geography to make their tools work.

## Yes, I know it's a label

Calling a signing region `us-east-1` doesn't move anyone's servers to the United States, change where the disks sit, or change which jurisdiction they operate under. A Canadian name wouldn't prove Canadian residency either.

But being told to use `us-east-1` on infrastructure you operate feels like being handed an American flag and told it's the only one the software recognizes.

The region does have a job in the protocol. In [AWS Signature Version 4](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_sigv.html), it forms part of the credential scope and the signing-key derivation. A request signed for one region isn't automatically a valid request for another.

Self-hosted engines like [Garage](https://garagehq.deuxfleurs.fr/) let the operator configure that value. The endpoint tells the client where to connect, and the signing region tells it which value to use when authenticating.

An operator in that spot can change the server's signing region to match. Then every customer whose configuration already works has to change it too, to suit one client with no field for it.

## Compatible with whose infrastructure?

Amazon built S3, and the interface became useful well beyond Amazon, including to people running their own storage. A shared interface means a customer can choose storage and backup software separately. That choice depends on what the software lets you configure.

Treating `us-east-1` as unavoidable makes an American provider's naming convention a condition of participating in a supposedly shared interface.

I'd object to a mandatory Canadian label too. Where one operator hosts shouldn't become a naming requirement for anyone else.

The flag should be your choice, including a name with no national flag attached. Ours is `storm`.

## Sovereignty includes being able to use the thing

If a customer's existing tools can't express the signing region, that customer still can't use what was built. The cost lands somewhere. Either the customer changes software, the operator changes a working configuration, or they give up on connecting the two. A hardcoded default can exclude independent infrastructure without anyone intending it.

Sovereignty gets discussed through datacentres, procurement and ownership, but it also depends on whether ordinary software lets an independent service participate, down to whether a client has a region field.

Storm has compatibility limits too. Garage doesn't implement every S3 feature, and we document that. If a customer needs a feature we don't support, they deserve a clear answer. We expect the same precision from the clients connecting to us.

If you build a tool with an S3-compatible connection, the request is small. Give users a signing region field. Keep `us-east-1` as the default if you want, and let them replace it.

Your users already chose their storage, their software, and where their data would live. Accepting somebody else's flag shouldn't be the price of making that choice work.

{~ card ~}
[Storm Buckets](https://stormdevelopments.ca/buckets/) is S3-compatible object storage hosted in Canada, built on Garage. It's in open alpha, and the signing region is `storm`.
{~~}
