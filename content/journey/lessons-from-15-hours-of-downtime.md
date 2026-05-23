---
title: Lessons from 15 Hours of Downtime
published_at: 2026-05-19
description: "Both my providers went down simultaneously because they shared a datacenter one layer below the surface. Layered coupling is the hardest kind to see."
quote: I thought I had two providers. I actually had one datacenter, hidden one layer down the stack.
tags:
  - self-hosting
  - infrastructure
  - canadian-digital-sovereignty
  - post-mortem
---

I noticed stormdevelopments.ca was down at 10 AM today. Tried to SSH into the Canadian Web Hosting VPS - connection refused. Tried to SSH into my Garage node, which was hosted directly with eSecureData (a different provider) - also connection refused.

That's the moment I knew something was wrong beyond a single VPS dying.

Then I checked northtube.ca - down. Storm Forge, my self-hosted git - down. Every server I had on either CWH or eSecureData was unreachable.

The thing that bothered me wasn't that CWH was down. The thing that bothered me was that my Garage node, on a completely different provider, was also down. I had specifically used two different VPS companies because I thought that was diversification. So why were both dead simultaneously?

The answer came an hour later from a Reddit thread: eSecureData isn't just where I had my Garage node. It is also the datacenter Canadian Web Hosting uses. CWH's servers and my Garage node, all in the same Vancouver building. When eSecureData had power events at the rack, everything I owned went dark at once.

I thought I had two providers. I actually had one datacenter, hidden one layer down the stack.

## What Was Actually Broken

- stormdevelopments.ca (marketing site, all the blog content, the project I'm building)
- northtube.ca (PeerTube, every video, the federation)
- git.stormdevelopments.ca (Storm Forge - my source code)
- My Garage node at eSecureData
- The CWH ticket portal (so I could not even file a complaint)
- CWH's own marketing site (so I could not even verify the outage was theirs)

What still worked:

- Social Ontario, my Mastodon instance, because I had hosted it somewhere else on a hunch
- Local clones of my git repos, because I am a creature of habit who pulls before editing

That second list is the only reason I got any work done today.

## The Actual Lesson

The easy version is "I needed two providers and I should have had two providers." The harder version is "I had two providers and it did not matter."

This is the layered coupling problem. You decouple at the most visible layer - different VPS companies, different marketing sites, different invoices - and you feel like you have done the responsible thing. But the actual coupling lives in the layer below. Different providers can share the same datacenter operator, the same network upstream, the same physical building. Your "diversification" is only as diversified as the deepest shared dependency.

For me, the shared dependency was eSecureData. A Vancouver datacenter that both my VPS provider and my object storage provider quietly depended on. I never bothered to check, because the surface-level diversity felt sufficient. It was not.

If those services had been truly diversified - different providers, different datacenter operators, different cities - the outage looks completely different:

- Site dies, storage stays up. Applications using S3 keep working. I have somewhere to post "we are aware of this and working on it."
- Storage dies, site stays up. I can post a status update, email people, explain what is happening.
- Both die independently for unrelated reasons. Still beats "everything dies at once because they share a building."

This was not a sovereignty failure - CWH and eSecureData are both Canadian. It was a coupling failure that had been invisible to me until something stressed the system. Most of the worst coupling is invisible.

## What I Did

Mid-outage, while still pissed and unable to think clearly, I started shopping for a new hosting provider. Almost bought a 36-month plan with a different Canadian host before catching myself. Long-term commitments made in the middle of a crisis are almost always wrong.

I ended up signing up for a storage VPS at ServaRica, a small Montreal-based Canadian hosting company. Different city. Different datacenter operator. Different provincial power grid (Hydro-Québec instead of BC Hydro). The kind of diversification that is actually diversification, not just two surface-level providers sharing a building.

ServaRica has been quietly running infrastructure for Canadian self-hosters since 2010. The LowEndTalk crowd has recommended them for over a decade - which is the strongest legitimacy signal in the small-hosting market. They own their hardware, run on hydroelectric power, and their pricing for storage VPS is genuinely better than I expected.

That ServaRica VPS becomes the first node of what will eventually be a multi-region cluster - replicated across multiple Canadian cities, different providers, different datacenter operators. The plan was always to do this. Today made it the priority instead of "next month."

stormdevelopments.ca will probably stay on CWH for another month. I paid the bill two days ago and there is no point cutting that short. But the moment the renewal comes up, or the moment they have another outage like this, the site moves to a different provider on a different datacenter operator. The site and the storage will never share a single point of failure again - especially not an invisible one.

## If You've Been Thinking About a Canadian VPS

{~ card ~}
ServaRica is genuinely worth a look. They give you 3TB of storage that grows about 1TB per year automatically (kind of like how Gmail's inbox size grew over time, but for disk), and their support actually answers tickets. Here's my referral link: **[clients.servarica.com/aff.php?aff=1318](https://clients.servarica.com/aff.php?aff=1318)**

I get a small commission if you use it. I'd recommend them either way - they're the provider I just bet my own infrastructure on - but if you're signing up anyway, you might as well send a couple bucks toward the guy who burned a Tuesday on this.
{~~}

---

{~ alert type="info" ~}
Note to self: diversify your decoupled services - and check what they share in the layer below.
{~~}
