---
title: What Your Pipeline Pulls
published_at: 2026-06-26
kind: newsletter
issue: 3
featured: false
cover: /media/images/newsletter/what-your-pipeline-pulls-cover.png
cover_alt: "Own Your Stack, Issue 3 - What Your Pipeline Pulls. Title over a circuit-board background."
description: "Move your forge and the next question is where the pipelines run. CI executes your code on top of a stack of other people's, all of it holding the same keys - and a version tag promises far less than it looks like it does."
quote: A floating reference is a standing invitation to run code you have never seen.
tags:
  - ci-cd
  - supply-chain
  - forgejo
  - woodpecker
  - sovereignty
---

Every CI config has a line like `uses: actions/checkout@v4` in it, and it's the one nobody reads twice. On every push it pulls code you've likely never read and runs it on a machine holding your production secrets.

We've spent this series bringing your stack under your own control, one layer at a time. Last issue it was the forge: your source and its full history on infrastructure you run. CI is the next layer up, and it's the one that quietly hands that control back, because a pipeline runs your code on top of a stack of other people's, and all of it executes with the same keys.

## Where the Workflows Go

Move your forge and within a day you hit the next question: where do the pipelines run? The forge holds your code, but the thing that builds and tests it on every push was, on most teams, GitHub Actions - and that doesn't come with you.

Forgejo ships its own answer. Forgejo Actions speaks the same workflow syntax as GitHub Actions, down to the `uses:` line, and runs it on a runner you host. Anyone fluent in GitHub Actions reads a Forgejo workflow cold. You keep your pipelines and change where they execute, not how they're written. That's the whole pitch, and it's a good one.

That convenience carries a habit you may not have meant to pack. `uses: actions/checkout@v4` resolves to github.com, and so does the rest of the marketplace - even GitHub's own first-party actions live there. Move your forge to a Canadian VPS and a workflow written the usual way still reaches back to Microsoft on every run to fetch the actions it leans on. Most people stick with the marketplace because it's the path of least resistance and every template opens with it. Forgejo Actions inherits that path, but it doesn't force you down it.

![Comparison of self-hosted CI options for Forgejo: Forgejo Actions versus Woodpecker.](/media/images/newsletter/inline/woodpecker-ci.png)

*(Reference: linuxiac.com)*

The other option comes from an older line. Woodpecker is a community fork of Drone, an early container-native CI engine from 2012. Harness bought Drone in 2020 and folded it into a commercial platform; the original is a frozen snapshot now, with users pointed at a rewrite. In 2022 the people who wanted it to stay open forked it, and that fork is Woodpecker: Apache-licensed, community-run, still doing the one job. It has no marketplace and no `uses:`. Every step names a container image and the commands to run inside it, a plugin is just a container, and nothing in a pipeline reaches for outside code unless you wrote the line that reaches.

So the choice is real, though narrower than it first looks: both can be fully sovereign, and the difference is in what keeps them that way. On Forgejo Actions, staying off the GitHub marketplace is a discipline you hold line by line. With Woodpecker it's structural, because there's no marketplace to reach for at all. The effort runs the other way: Forgejo Actions is bundled into the forge you already run, while Woodpecker is a separate server and agent to stand up and keep alive.

## What a Version Tag Actually Promises

Go back to that `uses: actions/checkout@v4`. The interesting part is the `@v4`. It looks like a version number, the way 2.4.1 is a version number: fixed, naming one specific thing. It isn't. A tag is a movable label, a pointer the action's owner can aim at any commit they like, whenever they like. `@v4` means "whatever the maintainer is currently calling v4," and that can change between this push and your next one.

That makes the trust you extend a strange kind. You don't re-read the action's code on every run. You fetch whatever `@v4` resolves to right now and execute it, with your deploy keys and cloud credentials sitting in the same environment. So trusting that action wasn't a decision you made once, the day you added the line. It's a standing grant, renewed silently on every push, to code that can differ from the code you looked at when you wrote it down.

There's a fix, and it's the kind nobody turns on until they've been burned. Pin to a commit hash instead of a tag: `uses: actions/checkout@<full-40-character-sha>`. A commit hash can't be moved; it names exact bytes. The cost is that updates stop arriving for free, because bumping the action now means changing the hash yourself, on purpose, after you've had a look. The other option is to skip the marketplace entirely and write the steps yourself. Either way you're trading convenience for control over what runs.

None of this is unique to GitHub Actions. Woodpecker has the same shape one layer down: `image: someorg/tool:latest` is a floating tag too, and `:latest` tomorrow is not promised to be `:latest` today. Pin the digest and the pipeline runs the bytes you picked. The rule outlives any single tool: a floating reference is a standing invitation to run code you have never seen.

## When the Tag Moved

In March 2025, someone noticed this flaw, and accepted the invitation. tj-actions/changed-files was a popular open-source action - it worked out which files changed in a pull request - and it sat in the workflows of more than 23,000 repositories. An attacker got hold of a token with write access to it and did the simplest possible thing: they moved the tags. Thousands of pipelines were repointed to a single malicious commit.

Nobody had to update anything, and that's the part worth sitting with. The next time each pipeline ran, on its next routine push, it fetched whatever the tag now resolved to and executed it, exactly as designed. The payload scraped the secrets out of the runner's memory and printed them straight into the build logs. On public repositories those logs were readable by anyone who cared to look: cloud access keys, GitHub tokens, signing keys, all of it. It's tracked as CVE-2025-30066, and the workflows that walked away clean were the ones that had pinned to a commit hash instead of a tag.

![The NVD entry for CVE-2025-30066, the tj-actions/changed-files supply-chain compromise.](/media/images/newsletter/inline/tj-actions-cve.png)

*(Reference: nvd.nist.gov)*

So the fix from the last section earned its keep. Pinning was the line between the repos that leaked their keys and the ones that didn't. But it only settles the mechanical question of what runs, and this incident turns on a harder one. The dangerous code was a trusted, popular, open-source tool that 23,000 repositories had chosen on purpose. The attacker had legitimate write access to it, taken with a stolen token, and never needed a flaw in the code at all. So the real thing under you, every time you lean on an open-source project, is the people who can change it, and whatever holds them accountable when they can't, won't, or get compromised.

{~ alert type="info" ~}
Next issue: the dark side of open source. Who's actually behind the dependencies you build on, what happens when a lone maintainer burns out or hands the keys to a stranger, and how to tell a project you can safely bet a company on from one you can't.
{~~}
