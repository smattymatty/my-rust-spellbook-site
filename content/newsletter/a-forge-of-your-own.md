---
title: A Forge of Your Own
published_at: 2026-05-28
kind: newsletter
issue: 2
featured: true
cover: /media/images/newsletter/a-forge-of-your-own-cover.png
cover_alt: "Own Your Stack, Issue 2 - A Forge of Your Own. Title over a circuit-board background."
description: "GitHub is fracturing, but the more interesting story is what was never there in the first place. The forge layer has an open answer already running 320,000 projects in production."
quote: Every other layer of your stack is replaceable infrastructure. The source code is the company.
tags:
  - github
  - forgejo
  - sovereignty
  - open-source
---

GitHub is fracturing. Zig left in December, Ghostty in April, both citing reliability. Three outages in January took the platform offline for a combined 14 hours - millions of projects unable to handle their core workflow. The platform 100 million developers depend on is changing under their feet, and most haven't decided whether they're staying.

The more interesting story isn't who's leaving. It's what was never there in the first place. Garage - the open s3 storage engine - lives on Deuxfleurs' own forge. PeerTube lives on Framagit, a community-run GitLab instance maintained by the nonprofit Framasoft. Neither has ever run their development workflow on GitHub. Both ship serious software that runs in production at scale. There has always been a parallel ecosystem of open source infrastructure that exists outside of GitHub. That ecosystem is only growing larger.

If you run an organization that ships code, this is the year that decision becomes yours. The good news is that the answer already exists, in production, hosting roughly 320,000 projects. I started running it last year.

The forge is called Forgejo - open source, self-hosted, AGPL-licensed, maintained by Codeberg e.V., a nonprofit in Berlin. Codeberg runs the flagship public instance. I run two of my own: a private one for my company's internal source, and gitforge.ca, a public generalist forge anyone can sign up for. Both on Canadian VPS hardware I operate.

![The Forgejo project website on codeberg.org, showing the open-source, self-hosted forge.](/media/images/newsletter/inline/codeberg-forgejo-website.png)

*(Reference: codeberg.org/forgejo/website)*

On the day-to-day, Forgejo works the way GitHub works: pull requests, issues, wikis, releases, container registries, built-in CI through Forgejo Actions. SSH and HTTPS push. Custom domains. Migration tooling that pulls your repos, issues, PRs, and contributor history out of GitHub in one command. The difference is where the bytes live, and who controls them.

## More Than Just Source Code

Canadian organizations have spent the last three years moving customer data onto Canadian servers. What didn't move is the place where the company itself is written down. The source code. The PR threads where decisions were debated. The issue history. The design documents. The architectural choices made and rejected. All of it sits on American servers, owned by Microsoft, governed by the US CLOUD Act.

Source code isn't an asset like a server is an asset. A server can be swapped for an identical one in another data center; the code can't. The code is the only thing that documents how your business actually works - how it ships features, how it handles bugs, how it integrates with the rest of the world. Every other layer of your stack is replaceable infrastructure. The source code is the company.

When you accept GitHub as the answer for the forge layer, you accept Microsoft as the keeper of the document of your company and the document of your career. For most of us, that keeper is now a division of Microsoft called CoreAI. The contract didn't have your signature on it.

## Sovereignty Doesn't Stop at the Database

Moving source code to Canadian infrastructure is the invisible part of the same work. The same questions apply: where do the bytes live, who can read them, what jurisdiction governs them, how recoverable is the data if the host has a bad week. The answers should be the same too. They're rarely treated that way.

A self-hosted Forgejo instance answers them. Where the bytes live: a Canadian VPS, by your choice. Who can read them: your developers and whoever you've authorized, full stop. What jurisdiction governs them: whichever province the VPS is in. How recoverable: a daily backup to an S3 bucket somewhere is twenty lines of cron.

The cost is real - someone has to run it. The cost is also bounded. Forgejo at small-to-medium scale runs on a VPS that costs less per month than a few GitHub Enterprise seats.

## For Engineers: Your Career Isn't a Hosted Service

A GitHub profile is a rented résumé. The rent buys real value - visibility on a platform 100 million developers visit, a stable URL recruiters bookmark, a contribution graph that signals consistency to people who care about that signal. The rent also has terms. When the platform changes those terms - the API access, the AI training defaults, the search ranking of your repos - your résumé changes with it.

The portfolio version is different. The portfolio version is a domain you own, a forge you control or contribute to, and a record of work that's verifiable wherever it lives. A PR merged to a project on Codeberg is a PR merged. A patch landed in a project hosted on the maintainer's own forge is a patch landed. Git is git. The signed commit you pushed to one forge looks identical on any forge that can read it.

The case for portability isn't that GitHub is going away tomorrow. The case is that the document of who you are as an engineer should not be one platform's terms-of-service away from changing form. Hosted profiles get demoted in search results, throttled by API changes, auto-opted into uses of their data the profile owner didn't choose. Those things happen on a regular schedule, even to the most-celebrated accounts on the platform. They don't have to happen to yours - if yours doesn't have to live there.

The portable version doesn't require leaving GitHub. It requires having a record of work that doesn't depend on GitHub. A domain at yourname.dev. A forge you contribute to that isn't owned by a trillion-dollar company. A signed commit identity that lets contributors verify your patches across forges. Pieces, not a profile.

## What's Happening at GitHub

In August of last year, GitHub was folded into Microsoft's new CoreAI division. CEO Thomas Dohmke left the same month. The platform that 100 million developers depend on stopped being primarily a developer platform and started being primarily an AI strategy.

The consequences have stacked up. April 2026 uptime came in at 86% - hours per day of blocked workflows across the platform. In February, GitHub confirmed approximately 3,800 internal repositories had been exfiltrated in a breach detected late last year. On April 24, the terms of service auto-opted every non-enterprise user into AI training on their Copilot interaction data - code, context, navigation patterns. Enterprise contracts were exempt; freelancers and small teams were not.

The most public response has come from Mitchell Hashimoto - GitHub user 1299, joined February 2008, founder of HashiCorp, creator of Vagrant, Terraform, and Packer. For the month before his April 28 farewell post, he kept a journal marking an X next to every date a GitHub outage blocked his work. Almost every day got an X. On the day he published the post, GitHub Actions was down for two hours. He summarized:

> I want to ship software, and it doesn't want me to ship software.
>
> — "Ghostty Is Leaving GitHub" by Mitchell Hashimoto

When the engineer who built Terraform announces he can't use GitHub for serious work, it's a data point about where the platform's priorities have moved.

The intermediate position in this conversation is GitLab - more open than GitHub, supports self-hosting, exposes more of its source. GitLab is also open-core, the structural shape we covered last issue with MinIO: a commercial company running an open-source project as marketing for a paid edition. Better than GitHub, not the same category as Forgejo.

## The Open Answer Already Exists

In the first issue we covered Garage - the open-source object storage engine, maintained by the French collective Deuxfleurs, demonstrated in production at scale. The forge layer has the same arrangement, in a different country. Codeberg e.V., a registered nonprofit in Berlin, maintains Forgejo and runs the flagship public instance, which hosts roughly 320,000 projects. These two organizations use the same license, similar governance, and are the same kind of proof.

The migration off GitHub is not a research project. The forge software exists. The hosting model is established. The migration tooling pulls your repositories, issues, pull requests, and contributor history out of GitHub in one command. The hard part isn't technical - it's the social-capital question, and that question has a workable answer too. You don't have to delete the GitHub mirror. You just have to stop treating it as the only place where your work is allowed to live.

My company's source lives on an internal forge. My open source contributions span three forges. The graphs on each are smaller than the unified one I used to maintain on GitHub. The work hasn't slowed down, only the destination has changed.

{~ alert type="info" ~}
Next issue: the CI/CD layer. Once the forge moves, the next question your team asks is where the workflows go. We'll cover Forgejo Actions, Woodpecker, Drone, and what each costs in complexity versus what each buys in independence.
{~~}
