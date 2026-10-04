---
title: "I Don't Let My Agents Commit"
published_at: 2026-10-03
carved: true
kind: post
description: "I have to understand my codebase, own it, and keep a hand on its design without drowning in what agents produce. So nothing lands until I've read it and run the commit myself."
quote: The human is the loop.
tags:
  - ai-assisted-engineering
  - git
  - handover
  - owning-your-tools
---

My agents don't run `git commit`. When one finishes a unit of work it tells me what it did, in a short numbered list with a file and line for each claim, and then it hands me one fenced block of shell. I call that the handover. I read the list, I read the block, and I paste it. The commit, the push and the merge are mine. I do it because the agents are faster than I am and I'm the one who has to understand the codebase when they're gone. The block is where I stop and check.

It started with agents overwriting each other. I ran several at once and branched the way I always branch, and the pull requests often came back with conflicts, and I'd open a diff and not remember touching the file. I hadn't. An agent had. Worktrees were the next experiment. Agents forgot to remove them, or collided anyway, or a session got cut off and the next agent never knew the work already existed in a tree somewhere. I pulled it back to one checkout, with one owner of its git state, me.

## The block

After a lot of iterations this is the shape I settled on, one chain of commands joined with `&&` so the first failure stops everything after it:

```bash
cd ~/Projects/website && \
{ [ "$(git branch --show-current)" = "main" ] || { echo "wrong branch"; false; }; } && \
git fetch origin && \
git merge --ff-only origin/main && \
git checkout -b relay-retry && \
git add -- src/relay.rs tests/relay.rs && \
git diff --cached --stat && \
git commit -m "Retry the relay once before failing" && \
git push -u origin relay-retry && \
git checkout main && \
git merge --ff-only relay-retry && \
git push origin main && \
git push origin --delete relay-retry && \
git branch -d relay-retry
```

Every line in there but one is a scar.

The guard is the commit that landed on another session's branch and reverted the tree when repaired. Two words and the chain stops. The braces keep a failed `cd` from saying "wrong branch" too, since `&&` and `||` bind at the same strength.

The fetch and fast-forward are a push refused mid-block because main moved while the agent worked. Catch main up first and the last merge stays a fast-forward.

The branch lives four seconds. Every change gets one, and a refused push to main still leaves the commit on the server under a name.

`--stat` is the one line that isn't a scar, the last look at what's staged before it's a commit, and Ctrl-C there costs nothing.

The two deletes are fifteen merged remote branches, one per handover, behind a clean local list. `git branch -d` only removes the local one, and the remote goes first so a failure is easy to retry.

And never an `exit`. The block runs in my shell, and `exit` closes the terminal whether or not the work landed. Once, a clean merge read as a crash. `&&` already stops on failure.

## Why I bother

This runs against most of the advice right now. A lot of people will tell you to never read the code. Run hundreds of agents, don't pay attention to them, trust it and don't worry. I refuse a lot of the AI hype cycle, and that part the most. I'll take what's useful from these tools, and the most useful thing they do for me is sharpen my understanding. A speedup that costs me that, or my hand on the design, is a trade I don't want to make.

The handover is a speed bump I put in on purpose. It makes me stop, once per unit of work, and get an idea of what the agent did and whether we're still on the right track. I don't read every line every time. I read enough to know what landed and why, and the commit has a person's name on it who can say what happened when something breaks.

It pairs with my other rule, which is no more than two sub-agents alongside the main one. Past that I'm triaging a feed. Cognitive overload is the thing I guard hardest against, because I want to understand this codebase and be in the room while it's designed, and I can't make a decision or hold a mental model while being told six things at once.

One thing at a time per repo doesn't mean one thing at a time. I have four repos that feed each other, all one system, and work can be moving in each of them while every checkout still has a single owner of its git state. That's where the interesting engineering problem moved. It's the architecture of the whole fleet now, which pieces can run in tandem, and which ones I need to be able to come back to and understand.

The one thing I've been experimenting with and have been willing to bend these rules on is dynamic workflows. What I'm trying right now runs writers one at a time in one checkout and reviewers in parallel, and it ends in the same handover block as everything else, so I count the whole run as one agent. That has changed how many I'm willing to have alive at once. It's the next post.

## Skills are the other half

[I've written before](/journey/grill-and-drill-skill.html) about the importance of editing installed skills to fit your workflow, because a skill is somebody else's judgment running in your assistant. The handover is the same idea at the other end of the work. Skills shape how the agent thinks on the way in. The handover decides what gets to become permanent on the way out, and nothing does that I haven't looked at. The block template lives in my CLAUDE.md, which every agent reads before it starts, so by the time one hands me a block I already know its shape and I'm only checking the parts that changed.

## The human is the loop

People say human in the loop like the human is a checkpoint the loop passes through. In my setup the human is the loop. I'm still solving the hard engineering problems and I still own the design. I grill the design before an agent touches a file, I write the brief it works from, I scope the tests and I read what comes back. The agents type in between, and then they stop, explain, and hand me a block. I decide if it lands.
