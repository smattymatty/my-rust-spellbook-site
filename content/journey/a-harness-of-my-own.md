---
title: "A Harness of My Own"
published_at: 2026-07-14
kind: post
featured: false
description: "Alberta published 21 white papers on turning an AI agent workforce loose on a 40-year, 466-million-line technical estate. Reading the first two, I found the architecture behind a framework I'd already built."
quote: They published the whole method, which means it isn't locked to the model they happened to run it on.
tags:
  - gooseloop
  - ai-assisted-engineering
  - canadian-digital-sovereignty
  - open-source
  - cohere
---

I build a program called gooseloop. It runs AI agents in a loop so I don't have to sit over them: I hand it a recipe, point it at a repo, and come back to work I can read and accept or throw out. I gave it a goofy name and filed it under tinkering.

Then, on July 6th, Alberta's Ministry of Technology and Innovation published [the Velocity White Papers](https://thevelocitywhitepapers.com/en/): twenty-one of them, open and free to read, on how a government is turning an AI agent workforce loose on a forty-year-old technical estate. I read the first two expecting a strategy deck. They read like an engineering manual, and the engineering was familiar.

## What the papers actually say

Start with the estate. Alberta runs about 466 million lines of code across systems that go back forty years. An AI agent can read all 466 million in under a day; no human team reads it in a career. Modernizing it would cost around $2 billion by their own low estimate, against a modernization budget of $80 to $120 million a year. That's roughly twenty years to rebuild if every other priority froze. And if you don't rebuild, only lifecycle what already exists at 2024 delivery speed, before agents, it takes 130 years. That 130-year figure is what the whole collection is built to disprove.

The problem has four heads and they name them plainly: technical debt, cybersecurity, growing demand, and a deficit budget. The papers are laid out as four escalating tiers of intervention, each more radical than the last, each aimed at the hydra.

What makes the first paper worth reading isn't the numbers, it's §04, where they describe the workforce honestly. Their agents are "genius amnesiacs" that forget the task between interactions. They are "unreliable witnesses," prone to distraction and flights of fancy. They are "sycophantic and fawning in their praise of even bad ideas." I have never read a truer description of these tools, and I did not expect it in a government document. The architecture that follows is one diagram: judgement stays with humans, agents do the volume. Every other paper in the set follows from that.

The second paper, on cybersecurity, is the one on my own terrain. Alberta blocks 189 million connection attempts a day, more than double what it blocked two years ago. A single government cyber incident runs $5 million a month in the best case. And the threat they spend the most words on is the supply chain: the Axios hijack, where a poisoned release of a package pulled 100 million times a week pushed remote-access trojans into developer machines and CI pipelines; a separate campaign that compromised over 5,500 repositories to harvest SSH keys, API tokens, and cloud credentials. Their response was to point agents at their own estate, where the agents "surfaced instances where staff had mistakenly committed secrets into the code repositories," then removed, cycled, and invalidated every one. They put the cost of finding and often remediating a single vulnerability this way at one to two dollars.

Three of the follow-on papers are the ones I'll be stealing from, because they're the implementation and not the argument. **Git Insights**: point AI at the entire git estate and let it report what's actually in there. **The Well-Built Harness**: enforce your standards through the AI delivery pipeline itself, so the machine can't ship work that violates them. And a paper on standing **red, blue, green, and yellow agents**, permanent adversaries and defenders run continuously against your own code.

## What gooseloop actually is

Gooseloop is small, and it splits cleanly in two.

The **core** is a framework that knows nothing about any domain. It has no concept of code, or docs, or security. All it owns is one fixed shape that every run passes through: **review, then body, then summary**. The review reads the current state and produces a plan. The body does the work that plan called for, one unit at a time. The summary writes down what happened. The framework enforces that order and the contract around it, and nothing else. It's generic on purpose.

Everything domain-specific lives outside the core, in a plugin the code calls an **engine**. An engine is two objects and a folder of recipes:

- The **Environment** is the nouns: what this loop is allowed to see. A repository, a config file, a document map, a journal. It declares the inputs, and nothing the engine can't name in its Environment reaches a model.
- The **Engine** is the verbs: what to do with those nouns, and how the work gets decided.
- The **recipes** are the actual instructions handed to the model, written as YAML, one per phase. They're procured and layered rather than edited in place, so a machine-specific tweak lands in an overlay file and never touches the shared recipe.

Core stays generic; an engine plugs into it through those two objects and the recipes. That's the whole extension surface. It's why I can add a new loop without opening the framework, and why the framework can't grow a dependency on any one engine.

One axis decides an engine's character: **who chooses the work**. In a model-driven engine, the review looks at the state and the model decides what the body should do. In a deterministic one, the engine computes the work itself, in plain code, and the model's only job is to execute each unit, never to select it. That is the cost-and-trust dial. The more the machine decides, the more you're trusting a genius amnesiac's judgement. The more you decide in code, the less you're paying a model to do arithmetic it will sometimes get wrong.

The engine I lean on hardest checks documentation for drift. It holds a map: which source of truth each published doc or page is derived from. Cheap, deterministic code decides what actually moved, a price in the code that a live marketing page still contradicts, a promise in the privacy policy that no retention job enforces. Only the things that genuinely drifted reach a model, and all the model does is draft the correction. That draft is written to a file for me to read and apply by hand. The map that decides what gets checked is never written by the machine: the engine can propose an addition to it, as a change for me to make, but it cannot make the change itself. The agent drafts, and I decide whether the draft lands.

Under all of that, every model spawn runs inside a sandbox, and gooseloop scans the output for anything shaped like a leaked credential before it goes anywhere.

## Same doctrine, different scale

The parallels are specific:

- Their §04, the genius amnesiacs and unreliable witnesses, is the entire reason the Environment and the recipes exist. An agent with no memory is only as good as the state and instructions you hand it each turn, so gooseloop makes those a declared, inspectable object instead of a prompt I retype and hope I got right.
- Their judgement-human-agents-volume diagram is my review/body/summary with a human seal on the end. Nothing an agent produces applies itself.
- Their Git Insights, pointing AI at the whole estate, is what an engine does to a repo, and I have my own estate to aim it at: everything I self-host.
- Their Well-Built Harness, standards enforced through the pipeline, is gooseloop's contract plus the fitness checks I already run: an import-linter that fails the build on a topology violation, a type gate, a precheck that refuses to start a malformed run.
- Their one-to-two-dollar remediation is the deterministic-before-model rule: if plain code can decide something, the model never gets asked.

The honest difference is scale and standing. Alberta runs fleets, one to ten agents per application, permanently watching. I run one pass at a time, and my always-on watcher is a separate tool entirely. But the doctrine underneath is the same, and I didn't take it from them. I read a ministry describe, as a survival requirement, the thing I'd been building because it was the only way handing work to an agent felt safe.

## The dependency the cure creates

The papers name the enemy as dependency: vendor lock-in, out-of-support tech, foreign code in the supply chain, with one paper proud of tracing open-source contributions back to foreign countries and cutting them out. Then the cure runs on American AI, on frontier tiers including one whose access the US recently froze for foreign users. You can modernize every system Alberta owns and still not control the result, because the intelligence doing the modernizing answers to a company in another country that has already shown it will shut foreign access off.

There are three labs at the frontier and all three are American. For a government that has to move now against a real threat, using them is the only option on the table, and I run on the same providers. What Alberta got right is the rare part: they published the whole method, in plain language, architecture and numbers and the genius-amnesiac paragraph included. Published that openly, the method isn't tied to the model that ran it. You can point it at any capable AI and it still works, and that is the only reason I get to be hopeful here. I built my last project's search on Cohere because it's Canadian, and I'm building the harness so it's ready the day a Canadian model can run the reasoning underneath it.

A province spent $100 million and eighteen months to write that shape down. I got to it on my own. The shape is cheap and anyone can build one. The expensive, borrowed part is the frontier model underneath it, and that is the part I'm still waiting on a Canadian answer for.
