---
title: Quebec Got Privacy Right - Why Law 25 Is My Gold Standard
published_at: 2026-06-25
kind: post
featured: false
description: "Quebec's Law 25 is the strictest privacy law in Canada, and the standard I hold Storm Buckets to even though PIPEDA is the one I'm held to. Confidentiality by default, readable consent, and the right to walk away with your data."
quote: You don't switch protection on. It's already on, and the burden is on the business to earn its way past that.
tags:
  - privacy
  - law-25
  - pipeda
  - canadian-digital-sovereignty
  - storm-buckets
---

Most of my side projects were built to hold nothing. They run on your own machine, they talk to no server of mine, and the only database is a file on your own disk that I will never see. That's deliberate. I built them that way because I didn't want to be responsible for anyone's data. If you never collect it, you never have to guard it. Designing around having nothing to protect was the cleanest kind of safe.

The trouble is when you move beyond toy projects and start building a real platform. When I started building [Storm Buckets](https://stormdevelopments.ca/buckets), the bag stopped being empty.

It's S3-compatible object storage. Real people put real files in it, on servers I run, reachable wherever they need them. There is no version of that which holds nothing. The question I had spent years designing my way around became the only one that mattered: what do I actually owe the person whose data this is? Caring about privacy stopped being a feeling I could have and started being a thing I had to do on purpose. When I went looking for what doing it properly meant, the clearest answer in the country came out of Quebec: Law 25.

## The instinct still holds, it just costs more

Privacy-first design doesn't stop being the goal once the bag isn't empty anymore. I still reach for it. The principle is the same one that made the toy apps safe: collect the least you can, keep it the shortest time you can, and make the most private option the default. None of that changes. What changes is that it stops being free. Every one of those choices used to happen by accident because there was nothing to collect. Now each one is a decision I have to make on purpose, defend, and live with, because there's a real person on the other end of it.

So the work moves earlier. Before I build anything that touches personal information, I have to answer four questions about it:

1. What am I collecting,
2. why,
3. how long do I keep it,
4. and who can see it.

A field with no good answer to "why" gets taken out of the design. Not collecting something is always cheaper and safer than collecting it carefully. That's the whole instinct from my old toy apps, carried forward into a place where it now takes deliberate effort instead of falling out of the architecture for free.

What I didn't expect was that the list had already been written. [Quebec's Law 25](https://www.cai.gouv.qc.ca/), the strictest privacy law in the country, reads less like a compliance burden and more like a set of guidelines any service that respects the dignity of its customers' data should want to follow anyway. The idea underneath it is that your privacy is the default, and a company has to justify any move away from it. You don't switch protection on. It's already on, and the burden is on the business to earn its way past that. That instinct, privacy as the baseline rather than the upsell, runs all through the law.

## What Law 25 actually gets right

A few provisions stand out, and they're the ones I'd want to live up to even if no law required them.

The first is [confidentiality by default](https://www.legisquebec.gouv.qc.ca/en/document/lc/P-39.1?langCont=en#se:9_1): any technological product or service offered to the public that collects personal information has to use the highest privacy settings automatically, with no action needed from the person. That's section 9.1 of the act, and it's the baseline-not-upsell idea written into law. Most software ships with the data taps open and buries the shutoff three menus deep. Law 25 says the taps start closed and you have to ask before opening them.

The second is consent that a human can actually read. Law 25 wants consent requests in clear, plain language, asked separately from everything else and granularly per purpose, not smuggled into a thousand-word terms-of-service nobody reads. If you have to hide what you're doing inside legalese to get a yes, you didn't get a real yes.

The third is the one I care about most, because it's the same thing I believe about infrastructure: [data portability](https://www.legisquebec.gouv.qc.ca/en/document/lc/P-39.1?langCont=en#se:27). Law 25 gives people the right to get their own data back in a structured, commonly used format, that's the third paragraph of section 27. The point of that is the freedom to leave. A company that makes your data easy to take is a company betting on being good enough that you won't want to, and that's the only kind of lock-in I respect: none. It's the same reason Storm Buckets speaks the S3 protocol instead of some proprietary dialect. If you can walk out the door with your files at any time, I have to keep earning the reason you stay.

## Where I actually stand

Law 25 is a Quebec law. The law that governs me today is the federal one, PIPEDA, which has been the rule since 2000 and is the weaker standard. The federal picture is finally moving, [Bill C-36](https://www.parl.ca/legisinfo/en/bill/45-1/c-36) was tabled in June 2026 and would give a real regulator binding orders and serious penalties, but it's at first reading, likely a year or more from being law, and PIPEDA still governs until then. So aiming at Law 25 is a choice. It's the strictest bar in the country, and I'd rather build to the standard I admire than the one I'm held to.

I hold Storm Buckets to Law 25 as the bar, and the [privacy policy](https://stormdevelopments.ca/privacy) is where I keep an honest, current account of how close it actually is. Today that means operating under PIPEDA while working toward full Law 25 alignment. The policy is the live version of that story, not this blog post.

One thing worth being precise about, because it's where most Canadian-hosting pitches overclaim: Law 25 is not a data-localization law. It doesn't say your data must live in Quebec or even in Canada. What it does is require real scrutiny before personal information crosses a border, and a transfer to Ontario counts as crossing one. Keeping the storage in Canada doesn't satisfy a rule that doesn't exist. What it does is shrink that scrutiny to almost nothing, which is a real benefit, just not the one the marketing usually claims.

## Why this is the standard

Privacy-first was easy when I had nothing to lose on your behalf. The moment that changed, I had two options: treat privacy as a feeling I could keep claiming, or treat it as a list of things I actually do. Law 25 is the best version of that list I've found, written by people who start from the assumption that the person's dignity comes first and the company has to earn every exception. That's the standard I'm building toward, out loud, with the gaps named instead of hidden.

If that's the kind of infrastructure you want your own data sitting on, [Storm Buckets](https://stormdevelopments.ca/buckets) is the thing I'm building, it's honest about where it stands, and you can always walk away with everything you brought.