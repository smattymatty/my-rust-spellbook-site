---
title: "us-east-1: The Bane of Sovereignty"
published_at: 2026-09-17
kind: post
featured: false
description: "A backup client's hardcoded us-east-1 locked out our Canadian storage. S3-compatible tools should let users set the signing region."
quote: The flag should be your choice, including a name with no national flag attached. Ours is storm.
tags:
  - canadian-digital-sovereignty
  - storm-buckets
  - object-storage
  - garage
  - interoperability
  - self-hosting
---

One of our customers couldn't connect their backup software to our storage. I installed the Windows client in August 2026 to investigate. Our storage runs Garage with the signing region set to storm, and every request I examined was signed with us-east-1. I couldn't find a setting to change it, and an endpoint hostname containing storm didn't change it either.

The product hardcodes us-east-1 as its only region. When I asked the company, they replied that they're aware of the issue and working on it. I appreciate that response, but the incident still left me asking why an S3-compatible client would make one provider's region name unavoidable.

The region has a job in the protocol. In [AWS Signature Version 4](https://docs.aws.amazon.com/IAM/latest/UserGuide/reference_sigv.html), it forms part of the credential scope and the signing-key derivation. A request signed for one region isn't automatically a valid request for another.

Self-hosted engines like [Garage](https://garagehq.deuxfleurs.fr/documentation/reference-manual/configuration/) let the operator configure that value. The endpoint tells the client where to connect, and the signing region tells it which value to use when authenticating.

Being told to use us-east-1 on infrastructure you operate feels like being handed an American flag and told it's the only one the software recognizes.

## Compatible with whose infrastructure?

For my company, Storm Developments, the choice is Canada. Yours might be somewhere else, in your own country, through a local cooperative, or on a machine you own. Nobody should need to adopt somebody else's geography to make their tools work.

A shared interface means a customer can choose storage and backup software separately. Treating us-east-1 as unavoidable makes an American provider's naming convention a condition of participating.

I'd object to a mandatory Canadian label too. Where one operator hosts shouldn't become a naming requirement for anyone else.

The flag should be your choice, including a name with no national flag attached. Ours is storm.

## Sovereignty includes being able to use the thing

An operator can change the server's signing region to match. Then customers whose configurations already work have to change them too, to suit one client with no field for it.

Otherwise, the customer changes software or gives up on connecting the two. A hardcoded default can exclude independent infrastructure without anyone intending it.

Sovereignty gets discussed through datacentres, procurement and ownership, but it also depends on whether ordinary software lets an independent service participate, down to whether a client has a region field.

Storm has compatibility limits too. Garage doesn't implement every S3 feature, and we document that. If a customer needs a feature we don't support, they deserve a clear answer. We expect the same precision from the clients connecting to us.

If you build a tool with an S3-compatible connection, the request is small. Give users a signing region field. Keep us-east-1 as the default if you want, and let them replace it.

Your users already chose where their data would live. Accepting somebody else's flag shouldn't be the price of making that choice work.

{~ card ~}
[Storm Buckets](https://stormdevelopments.ca/buckets/) is S3-compatible object storage hosted in Canada, built on Garage. It's in open alpha, and the signing region is storm.
{~~}
