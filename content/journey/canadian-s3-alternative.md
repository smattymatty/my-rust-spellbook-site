---
title: Why I'm building a Canadian S3 alternative (The 85% problem)
published_at: 2026-06-21 
description: Three American firms hold 85% of Canada's public cloud. Storm Buckets is my open, Canadian-hosted, S3-compatible alternative for object storage.
quote: A Canadian flag on the login page fixes nothing if you still can't leave
kind: post
featured: true
cover: /media/images/blog/object-storage-in-canada.png
cover_alt: "Why I'm building a Canadian S3 alternative - Title over Canadian flags with Amazon, Microsoft, and Google logos."
tags:
  - canadian-digital-sovereignty
  - object-storage
  - storm-buckets
  - garage
  - s3-alternative
---

Three American companies hold eighty-five percent of Canada's public cloud. Amazon has forty-two percent, Microsoft thirty-one, Google twelve. That figure comes from ["Parting Clouds"](https://antimonopoly.ca/parting-clouds-creating-a-competitive-marketplace-for-compute/), a report the Canadian Anti-Monopoly Project published on June 2, 2026, and it runs well ahead of the roughly two-thirds those same firms hold worldwide. Canada is more dependent than the global average.

I've watched AWS tighten its grip on the internet for years, and that pull toward Canadian digital sovereignty is most of why I started [Storm Developments](https://stormdevelopments.ca/) in the first place. What the report gave me was the number. I knew the shape of the problem; eighty-five percent is the size of it, and it is worse than I had guessed. Canada sitting that far above the global average, paired with the last year of threats against Canadian sovereignty, is what pushed me from watching to building.

There are two ways to read eighty-five percent. One is the language of markets: concentration, switching costs, competition. The other is about power: whose law reaches your data, and who can decide to switch it off.

## The market reading

CAMP calls the Canadian cloud market broken, and the word from the report that stuck with me is "maplewashed." The warning is that a Canadian flag on the login page fixes nothing if you still can't leave. Lock the data in a proprietary format, charge egress fees that punish you for moving, expose APIs that no one else implements, and a domestic provider has just relocated the trap. What matters is not only who owns the servers but whether you can walk away from them at all.

Their prescription is the part most people skip past. They don't ask Canada to invent a national standard and wall itself off. They ask for commoditization: interoperability, portability, existing standards already in wide use, so that workloads move freely and no provider, foreign or domestic, can hold you in place. Treat compute like a utility. End the egress fees. Buy on terms that keep switching cheap. They are explicit that the danger is also a new domestic monopoly, a Canadian lock-in dressed up as sovereignty.

## The power reading

{~ alert type="warning" ~}
This is a sovereign risk as much as a competition problem. When the infrastructure your country runs on sits under another country's law, that law reaches your data.
{~~}

Under the United States [CLOUD Act](https://www.congress.gov/bill/115th-congress/house-bill/4943/text), American legal process can compel a US-based provider to hand over data in its custody or control, wherever the disks physically spin. Being hosted in Canada does not help you if your provider answers to Washington. The federal government's own numbers show the exposure: answering [a parliamentary question last year](https://www.cp24.com/news/canada/2025/09/19/national-defence-using-us-cloud-services-for-mission-critical-applications/), departments disclosed almost 1.3 billion dollars spent on US cloud since 2021, more than a billion of it to Microsoft alone, including applications National Defence calls mission-critical. The leverage that builds does not point toward Ottawa.

And the risk is not only that data gets read. It can be switched off. In January 2026, Cory Doctorow gave the keynote at a [closed Ottawa summit of federal technology chiefs](https://pluralistic.net/2026/01/29/post-american-canada/) and put the case to them directly: the same US firms that hold the country's data can be ordered to cut off access to it. He pointed to Microsoft cutting the International Criminal Court off from its own email and documents after US sanctions, and to a Microsoft France executive telling a French Senate inquiry he could not guarantee American authorities would never reach French data held on European soil, under the CLOUD Act, gag order attached. Build on that layer and a foreign government holds a switch over your systems, under a court order you never get to see.

## What I'm building

[Storm Buckets](https://stormdevelopments.ca/buckets/) is S3-compatible object storage, hosted in Canada, built on Garage, an open-source storage engine. S3-compatible is the deliberate choice the report points toward: a widely-used interface, so the command that moves you in is the command that moves you back out. rclone in, rclone out. I don't charge egress fees, so moving your data out costs you nothing and your reads are never a billing line. Open source is what lets you check the sovereignty claim instead of trusting it: you can read the engine, run it yourself, and owe me nothing if you would rather. The agent that runs the fleet, Storm Pulse, is open as well.

The CLOUD Act binds US-based providers. Storm isn't one. That doesn't make me untouchable, and I won't pretend it does: operators have root on the machines they run, which is exactly why anyone who needs protection from me, and not just from Washington, should encrypt their data before they upload it. On an open stack that costs nothing. What being Canadian does mean is narrower and real: there is no US parent company that American legal process can serve to reach your data behind your back.

{~ card title="Garage, by Deuxfleurs" footer="the open-source storage engine" ~}
Garage is the open-source engine under Storm Buckets, built by [Deuxfleurs](https://garagehq.deuxfleurs.fr/), a French non-profit collective that promotes self-hosting as an alternative to the hyperscalers. Their reason for building it is close to mine: the same concentrated power this piece is about.

People ask how French software makes anything Canadian-sovereign. They misread sovereignty. Open source belongs to whoever runs it, and I run Garage in Canada, on servers I operate myself. Sovereignty is about controlling where your data lives and keeping the freedom to leave. The best tool for that job happens to be open and French, and using it is how I own the stack.
{~~}

{~ accordion title="Two fixes I've merged into Garage" ~}
Running the engine in production means I hit its rough edges, and when I do I fix them at the source. 

[#1450](https://git.deuxfleurs.fr/Deuxfleurs/garage/pulls/1450) was a CORS fix so signed browser uploads work against buckets addressed by local alias, verified against Garage v2.2.0. 

[#1469](https://git.deuxfleurs.fr/Deuxfleurs/garage/pulls/1469) makes bulk delete report a missing key as deleted instead of an error, matching how AWS S3 behaves; it shipped with a regression test and landed in the v2.4 milestone. Both are merged upstream.
{~~}

I did not arrive here from theory. I run a video platform, NorthTube, that needed Canadian object storage, and there was no answer I trusted, so I built the answer. NorthTube is customer zero. I make the thing I needed and could not buy.

## The bet

One person with a few servers in Montreal does not move eighty-five percent, and I am not pretending otherwise. The point is that the alternative exists and stays open, with the exit built in from the start.

That is enough reason to build it. Infrastructure you can read, run yourself, and leave whenever you want should exist. So I build it in the open, with the receipts attached, and I keep building.

{~ card title="Storm Buckets" ~}
It's in alpha now. You can [see how it works](https://stormdevelopments.ca/buckets/), or read more about what I'm building at [Storm Developments](https://stormdevelopments.ca/). The stack is open source, so if you would rather not take my word for any of this, you can run it yourself.
{~~}

Mathew

---

## References

1. Canadian Anti-Monopoly Project. *Parting Clouds: Creating a Competitive Marketplace for Compute.* June 2, 2026. <https://antimonopoly.ca/parting-clouds-creating-a-competitive-marketplace-for-compute/>
2. Clarifying Lawful Overseas Use of Data Act (CLOUD Act), H.R. 4943, 115th Congress (2018). <https://www.congress.gov/bill/115th-congress/house-bill/4943/text>
3. The Canadian Press. "National Defence using U.S. cloud services for 'mission critical' applications." CP24, September 19, 2025. (Federal cloud-spending figure, from the government's response to Order Paper Question Q-94, submitted by MP Todd Doherty and tabled in the House of Commons.) <https://www.cp24.com/news/canada/2025/09/19/national-defence-using-us-cloud-services-for-mission-critical-applications/>
4. Deuxfleurs. *Garage*: S3-compatible object storage, AGPL-3.0. <https://garagehq.deuxfleurs.fr/>
5. Doctorow, Cory. "Disenshittification Nation." *Pluralistic*, January 29, 2026. (Keynote to the 2026 Digital Government Leaders Summit, Ottawa; transcript posted with permission of the Government of Canada CIO's office.) <https://pluralistic.net/2026/01/29/post-american-canada/>