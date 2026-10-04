---
title: The Internet Wasn't Inevitable
published_at: 2026-05-14
carved: false
kind: newsletter
issue: 1
featured: false
cover: /media/images/newsletter/the-internet-wasnt-inevitable-cover.png
cover_alt: "Own Your Stack, Issue 1 - The Internet Wasn't Inevitable. Title over a circuit-board background."
description: "Three layers of infrastructure had to fight to become open: Linux won, Django won, and now the storage layer is being decided."
quote: The fact that the modern web is mostly open is not a gift from the companies that owned most of it. It's the residue of a fight they lost.
tags:
  - infrastructure
  - open-source
  - sovereignty
---

In 1995, if you wanted to put a website on the internet, you would be forking over a lot of money in licensing fees for basic software like Operating Systems or Web Servers, and the company that sold them to you had final say over what you could do with them. The open internet you know today - the one where a teenager can spin up a server in a dorm room and reach a billion people - wasn't inevitable. It was the result of a specific bet that, at the time, almost everyone thought was foolish.

The bet was Linux: that an operating system built and given away for free could outperform what trillion-dollar companies were selling. The bet won.

If Microsoft or Sun had won the early infrastructure battle, the internet would not exist the way you know it. Not as a metaphor - as a literal statement about which servers would be allowed to talk to which clients, which protocols would be permitted, and which startups would have been licensed out of existence before they shipped. The fact that the modern web is mostly open is not a gift from the companies that owned most of it. It's the residue of a fight they lost.

## Layer One: Linux

You probably don't think about Linux. That's how completely it won. It runs your phone if it's an Android. It runs the server that delivered this newsletter to you. It runs the routers between your laptop and that server, every one of the world's top 500 supercomputers, and the in-flight entertainment system on the plane you most recently flew. Infrastructure is most successful when you stop noticing it.

![Bar chart: the global Linux operating system market growing from $7.64 billion in 2024 to $18.73 billion in 2029, a 19.8% CAGR.](/media/images/newsletter/inline/linux-market-2025.png)

*(Reference: The Business Research Company)*

The completeness of the win is the part that's easy to forget. In 2001, Microsoft's CEO Steve Ballmer called Linux "a cancer that attaches itself in an intellectual property sense to everything it touches." In 2014, the next Microsoft CEO stood on a stage and said "Microsoft loves Linux." Today, the Microsoft Azure fleet runs more Linux instances than Windows ones. The company that fought hardest to keep the open layer from existing now makes most of its cloud revenue serving it.

The reason it won is the reason it keeps winning whenever the same fight gets fought one layer up. Closed software has to be paid for, which means every line has to be justified to someone's quarterly numbers. Open software gets written by a coalition - sometimes hobbyists, more often paid engineers at companies that all depend on the same code and have decided cooperating on it is cheaper than each maintaining their own fork. Linux is maintained today by thousands of developers, the majority of them paid by companies like Red Hat, Google, Intel, and yes, Microsoft. The work is real work. The difference is that no single one of them owns it.

## Layer Two: Django

The same fight got fought at the application layer in the late 2000s, and the same side won. The expensive answer was a Java EE application server from Oracle, IBM's WebSphere, or Microsoft's .NET stack - software you paid licensing fees for, deployed onto hardware you also paid for, and could not modify without violating your support contract. The free answer was Django, a Python web framework released in 2005 by two newspaper developers in Lawrence, Kansas, who needed to ship local news sites faster than their deadlines allowed. They gave it away, letting it get used and improved by the community.

By 2012, Instagram was serving a hundred million users on Django. By 2019, it was serving a billion. The proprietary application servers that Oracle and IBM had spent decades selling to enterprise customers are still around, technically - you can still buy WebLogic if you want to - but no new company has started a serious project on one in years. The framework layer is open the same way the operating system layer is open: not because the closed vendors gave up the fight, but because the open option got so good that paying for the closed one stopped making sense.

![Diagram of Instagram's backend tech stack: Nginx fronting Django and Celery, all backed by Cassandra, PostgreSQL, Memcache, and RabbitMQ.](/media/images/newsletter/inline/instagram-backend-stack.png)

*(Reference: Ollayor's Blog)*

Two layers down, both fought and won. The third one is where we are now - and the closed vendor this time is not Microsoft or Oracle. It's Amazon.

## Layer Three: Where We Are Now

The closed answer is AWS S3, which is so deeply assumed as the default that "S3" has become a generic noun for object storage the way "Kleenex" is for tissues. It's a remarkable product. It's also a single American company holding the data of most of the internet, under American jurisdiction, at prices that change when the company decides they change, with a feature set that grows when the company decides it grows. For most of the past decade, the open answer was MinIO - until MinIO's company changed its license, gutted its open-source release, and effectively shut down the project in February 2026. Tens of thousands of organizations woke up to find that the open storage layer they'd built on was no longer open.

The answer that has been quietly waiting for that moment is Garage. It's an S3-compatible object storage engine, written in Rust, AGPL-licensed, designed from the start to run on heterogeneous hardware across multiple geographies - exactly the shape of infrastructure that small operators and sovereignty-conscious organizations actually have. It's built and maintained by Deuxfleurs, a French collective that uses Garage to host their own services. You can read every line of code that holds your data. If you find a bug, you can fix it - last week I submitted a one-line patch that was merged into the main branch within hours. The barrier to participating in the infrastructure you depend on is, for the first time at the storage layer, low enough to walk over.

The MinIO failure is the right thing to study before betting on the next open option. MinIO was a VC-backed company that open-sourced its product to drive adoption, then changed the license when that adoption didn't convert to enough revenue to satisfy investors. That's the open-core trap, and it's specific to a particular structure: an open project owned by a single commercial entity whose board has fiduciary obligations to capture more of the value it creates.

## The Pattern Itself

Garage is one specific answer, but the bigger answer is the pattern itself.

Every layer of the modern internet became open because some group of people built the alternative before there was a customer base waiting for it, and ran it themselves long enough to prove it worked. Linux ran on hobbyist machines for years before it ran the stock exchange. Django ran a few local news sites before it ran Instagram. Garage runs Deuxfleurs and, increasingly, the small infrastructure projects that have been quietly choosing it over the American defaults. The third layer is being decided right now. The companies best positioned for the next decade will be the ones that picked the open option early - not because it was morally virtuous, but because they were unwilling to bet the most valuable thing they own on a vendor whose terms they cannot change.

Linux took twenty years to become invisible. Django took fifteen to become inevitable. The storage layer is still negotiable.

{~ alert type="info" ~}
This newsletter is about what owning your stack actually looks like in practice - one layer at a time. Next issue looks at the fracturing of GitHub: the rise of self-hosted forges, what it means for your contributions when they don't all live in one place, and how to keep a professional record that doesn't depend on Microsoft maintaining your résumé for you.
{~~}

Welcome.
