---
title: The Dark Side of Open Source - Lessons Learned
published_at: 2026-08-14
kind: newsletter
issue: 4
featured: true
cover: /media/images/newsletter/the-dark-side-of-open-source-cover.png
cover_alt: "Own Your Stack, Issue 4 - The Dark Side of Open Source - Lessons Learned. Title over a circuit-board background."
description: "Open source is often the answer, but it's maintained by people you've never met. xz, event-stream, core-js and MinIO show what happens when the people behind your dependencies change."
quote: Before you bet on a dependency, don't ask what license it is. Ask who controls it, how many of them there are, and what happens when their incentives change.
tags:
  - open-source
  - supply-chain
  - governance
  - sovereignty
---

For three issues this newsletter has told you to move onto open foundations. Own your OS with Linux, own your framework with Django, own your storage with Garage, own your forge with Forgejo, run your own CI. Every one of those is the right call and I still stand behind all of them.

But there's a version of that advice that gets you burned. Open source is often the answer. It is also maintained by people you have never met, operating under structures you usually never check, and any one of them can change in a way that lands on your production system.

Last issue ended on the line that matters here: pinning a commit hash freezes the bytes, but it tells you nothing about who's behind them. This issue is that question. Who is actually behind the code you bet your company on, and what happens when they change.

These are the lessons, most of them learned the hard way by someone else, which is the cheapest way to learn them.

## xz: The Patient Backdoor

The one that should have been a catastrophe.

![Bitsight timeline of the xz incident: Jia Tan's contributions, patch introduction, malicious payload discovery, remediation.](/media/images/newsletter/inline/xz-timeline.png)

*(Reference: Bitsight)*

xz Utils is a compression library you've never thought about, maintained for fifteen years mostly by one volunteer, Lasse Collin. In early 2024 it was nearly used to backdoor most of the servers on the internet.

The attack was patience. A contributor calling themselves Jia Tan spent two years sending good patches and earning trust, while a rotating cast of other accounts, names never tied to a real person, pressured an overloaded Collin to hand off more control. He did. Then the backdoor went into the release tarball, not the git repo, so reading the source wouldn't have shown it to you. It linked into the SSH daemon and opened remote code execution for whoever held the key.

It was caught by luck. A Microsoft engineer noticed his SSH logins were running half a second slow and pulled the thread. The bad versions had only reached testing channels, never stable production, so the fleet survived. The internet was saved by latency annoying the right person on the right week.

You can't audit your way out of this. Trusting an open project means trusting whoever can change it, and one exhausted maintainer getting socially engineered into a handover is a failure no code review catches.

## event-stream: Handing Over the Keys

You don't need a state-level actor for the same shape of failure. You need one tired person and one polite email.

![npm tweet from Nov 28, 2018 announcing removal of flatmap-stream and event-stream@3.3.6 from the registry.](/media/images/newsletter/inline/event-stream-npm-tweet.png)

*(Reference: npmjs)*

event-stream was a popular Node.js utility, one of hundreds of packages built by Dominic Tarr, most of which he no longer used. In 2018 a contributor offered to take over maintenance. Tarr, reasonably enough, handed over publish rights to a package he had no stake in anymore. He did nothing wrong by the terms of the license, which like every open license disclaimed all responsibility.

The new maintainer made a few harmless commits, then quietly added a dependency carrying an obfuscated payload. It only activated inside builds of one specific Bitcoin wallet, where it stole the keys. It rode along undetected for over two months and was pulled down millions of times before anyone noticed.

Same lesson, smaller scale, no nation-state. A maintainer gave the keys to a stranger who asked nicely, because maintaining free software for people who never say thank you is genuinely exhausting and "here, you do it" is a very human thing to say.

## core-js: One Person, Half the Web

Sometimes nobody is malicious and you get burned anyway.

![The core-js logo.](/media/images/newsletter/inline/core-js-logo.png)

*(Reference: core-js)*

core-js is a polyfill library that, by its own maintainer's accounting, runs on more than half of the top ten thousand websites on the internet. It has effectively one maintainer, Denis Pushkarev.

In 2020 he went to prison for most of a year after a fatal motorcycle accident, and during his absence the project simply stopped. The funding, when he asked for it, came to a few hundred dollars a month for a dependency a large fraction of the web silently relies on. The whole industry kept increasing its dependence on a project that was one person's life away from freezing, and rated it healthy the entire time.

Nobody attacked anything here. The project was just one person, and the people depending on it chose not to see that. It is the most common version of this failure by far, and your own dependency tree almost certainly contains one right now.

## The corporate version

The three failures above are human: burnout, a bad handover, a lone maintainer's bad year. The corporate version is structural and it doesn't require anyone to burn out. It requires a single company to own the project and an incentive to capture more of it.

![The MinIO logo above an archive box labelled ARCHIVED.](/media/images/newsletter/inline/minio-archived.png)

*(Reference: itsfoss)*

MinIO is the case this newsletter has already studied. A VC-backed company open-sourced its object store to drive adoption, and when that adoption didn't convert to enough revenue, it relicensed, stripped features out of the community edition in 2025, stopped publishing community builds, and finally archived the repository.

Redis, Terraform, and Elasticsearch all did versions of the same move, relicensing from genuinely open terms to source-available ones the moment cloud competition threatened the business. In every case the community's escape hatch was the same: a neutral fork that no single company could relicense. Valkey for Redis. OpenTofu for Terraform. OpenSearch for Elasticsearch.

This is why alternatives are important. A project that is open today can stop accepting your contributions tomorrow and put a paywall in front of the next version the day after. The license is a snapshot. The trajectory is the story.

## Reading a project before you bet on it

These are the lessons learned, arguments for reading Open Source projects correctly, because the alternative is a closed vendor whose terms you cannot change at all. Open source remains the answer, you just have to know what you're looking at to avoid the pitfalls.

The biggest yellow flag is a single maintainer. One person is a single point of failure for security, for succession, for burnout, and for the polite-email handover that took down event-stream. Most good things start with one person, so a solo project isn't disqualifying. What matters is knowing you're carrying that risk and having a plan for the day the one person stops.

The other flags follow from the stories above. A contributor agreement that assigns copyright to a company is the mechanism that makes every corporate relicensing possible, so it's worth knowing who owns the copyright before you build on something. A declining release cadence, contributions sitting ignored, or a brand and domain held by a single commercial entity all point the same direction. A license that has drifted from permissive toward source-available is not automatically bad, but it is always a signal that someone is thinking about capture.

What I want to see instead is plural, neutral ownership. The reason I run Forgejo is not only that it's good software. Forgejo exists because the Gitea community watched their project get pulled into a for-profit company and forked it under a nonprofit specifically so that could never happen to them again. That is the green flag made visible: governance designed to prevent the exact trajectory this whole issue is about. Garage is the same shape at the storage layer, built by a nonprofit association that runs the software for its own use, with no investors to answer to and therefore no rug to pull. Foundation-stewarded projects work for the same structural reason: when no single company can unilaterally change the deal, the deal holds.

So the test is short, and it's the thing to carry out of this issue:

Before you bet on a dependency, don't ask what license it is. Ask who controls it, how many of them there are, and what happens when their incentives change.

Pin your hashes. Then go read the governance.

Next issue: the AI layer. The model is the engine, but the harness is the car, and the car is where the lock-in lives.
