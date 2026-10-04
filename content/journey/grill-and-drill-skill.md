---
title: "Grill and Drill with Skill"
published_at: 2026-06-30
carved: false
kind: post
description: "A skill you install without reading is shaping how you think, authored by someone else. I took Matt Pocock's grill-me, bent it to fit my head, and then built the inverse skill - drill - that the original never reached. The case against copy-paste, one layer up."
quote: The drill came from refusing to stop at the grill.
tags:
  - skills
  - ai-assisted-engineering
  - grill
  - drill
  - owning-your-tools
---

Every skill you add to your AI assistant is a small transfer of judgment. Somebody decided how a piece of thinking should go - what gets asked, in what order, when to stop - and wrote it down so your assistant runs it instead of improvising. Install one without reading it and you've accepted all of those decisions sight unseen, including the ones that fit the author and not you.

The good news is that the lock on that door is barely a lock at all. A skill is just markdown, and the thinking inside it is just words. There's no compile step, no API to learn, no version resolver deciding what you actually got - none of the friction that keeps you from cracking open a package off npm or pip and changing it. You open the file, you read it, you rewrite the parts that were written for someone else's head. The only thing standing between the skill the author shipped and the skill you actually want is whether you bother to read it.

## The install-and-forget habit

So the worst thing you can do with a skill is the easy thing: run the install command, get someone's command, start using it, and never open the file. It's the same reflex that has us all running dependencies we've never read, and most of the time that reflex is fine. A library does a mechanical job the same way in everyone's hands. You give it bytes, it sorts them, and it doesn't matter whose attention span the author had.

A skill is the case where it does matter. The author tuned it to their attention span, their team, their codebase, their sense of a good question. Take it unread and you've adopted all of that along with the useful part, and you'll never notice, because an unread skill works well enough to never make you ask why it's shaped the way it is.

## Grill, and what I changed about it

The skill I started from is Matt Pocock's [grill-me](https://www.aihero.dev/skills-grill-me). The idea behind it is genuinely good: instead of asking the assistant to plan something for you, you make it interview you, one question at a time, walking down every branch of the decision tree until the two of you actually share an understanding of what you're building. He later extended it into [grill-with-docs](https://www.aihero.dev/grill-with-docs), which reads a glossary and a set of decision records as it goes, sharpens fuzzy terms against them, and writes the resolutions back down so the next session starts further ahead. I use the bones of both every week.

I changed three things, and none of them are cosmetic.

The first: multiple choice over open prose. When the answer to a question is a choice between two or three named options, the skill hands me buttons instead of a blank. Picking the recommended option costs me nothing - I can accept by default and keep moving. Typing a paragraph costs energy I'd rather spend on the questions that actually need a paragraph.

The second: a progress bar on every single reply. A filled-and-empty bar, a count of questions left, a tally of docs touched. It tracks how close the design tree is to resolved, not how many questions I've answered. It sounds like a toy. It is the opposite of a toy for the kind of brain that loses the thread of a long session. The visible pile growing is the thing that keeps me in the chair.

The third, and the one the other two serve: many small questions beat a few big ones. A twelve-part question with sub-bullets is a wall I bounce off. Five one-line questions in a row is a rhythm I can stay inside. So the skill is forbidden from stacking two questions in a turn, and it's told to keep each one to a sentence or two and save the reasoning for after I've answered.

These are not improvements to grill in the abstract. They're grill bent to fit me. Someone with a different attention span would bend it somewhere else, or not at all.

![A grill session in progress: a progress bar reading 38% with about four questions left and three new docs touched, above a single multiple-choice question about a sim/render boundary with three named options and a Rust/JS code snippet.](/media/images/blog/inline/grill-example.png)

## Drill, the part that wasn't there

Here's where adapting turned into building.

The longer I used grill, the more I noticed it only ran in one direction. Grill pulls decisions out of your head and writes them down - into a glossary, into a decision record. It's generative. It produces the canon. But once the canon exists, nothing in the toolkit checks whether it ever made it back into the one place that matters: your head, the place a customer's question lands.

Reading a doc doesn't put it there. Take an architectural invariant you set yourself - some rule about what's allowed to cross a boundary. You wrote it. You'd know it on the page in a heartbeat and nod along. That's recognition, and it's cheap. Recall is the other thing: producing it cold, with its edges and the failure it's guarding against, when someone asks. The two feel like the same knowledge right up until someone makes you do the second one, and only one of them holds when the system is on fire.

So I built the inverse skill. Drill takes what's already written down and tests whether a head actually holds it, by making that head solve tight, constrained problems with answers derivable straight from the source. No open-ended discuss-the-tradeoffs - a concrete problem, and "what's the answer." The struggle is the whole point: if the skill explained the answer before I attempted it, I'd parrot it back and learn nothing.

The piece I'm proudest of is what happens when my answer and the doc disagree. The easy design assumes the answer key is right and marks me wrong. Drill refuses to. A divergence is ambiguous on purpose: maybe my mental model drifted, or maybe the doc went stale while the system moved on under it. The skill forces that fork into the open and makes me call it - my error to relearn, or a stale doc to fix. A quiz that always trusts its own key can't catch a rotten key. Mine has to ask.

And the fork has teeth on exactly this kind of rule. Fumble the edges of an invariant you designed and the drill makes you call it: did your own grasp of it drift, or did the system move and leave the doc describing a boundary that no longer exists? One answer sends you back to relearn the rule. The other sends you to fix a doc that's quietly lying about how your own system is wired - and a doc that lies about a security boundary is worse than no doc at all. Either way you found it at a desk instead of in a postmortem. Grill wrote the invariant down months ago. Drill is the thing that checks whether writing it down was ever enough to hold it. Usually it isn't.

The two run as a loop. Grill at the front of a piece of work to decide and record; drill at the back to verify the record actually landed. The docs are the shared ground between them. One of them did not exist until I stopped treating the other as finished.

## Why the copy is the anti-pattern

If I'd run the install command and used grill-me as it shipped, drill would not be a worse version of itself. It would be nothing. The whole second skill is the residue of asking, over and over, why is this mechanism here and what does it not cover - and you only ask that when you're rebuilding a thing, not when you're consuming it.

That's the case against copy-paste, and it isn't about credit. Pocock's idea is the seed of everything here and I'll say so anywhere. It's that a skill you installed unread is a set of answers to questions you never watched anyone ask. The fit to your actual work is accidental at best. You inherit the original author's constraints and call them your own, and you never find the gap their version doesn't reach, because finding it requires the one thing installing skips: taking the thing apart to see why it's shaped the way it is.

I build a company on the premise that you should own the layers you depend on rather than rent them from someone who can change them under you on their own schedule. A skill you loaded and never read is exactly that, one layer up: a black box shaping how you think, authored by someone else, free to move whenever they push an update. Your own reasoning process is a strange thing to hand to a label that can move.

So take the parts that are good. I took the Socratic spine and the write-it-down discipline, and I'm glad I did. Then make them yours - bend them to your own head, and keep going past the point where the original stopped. The grill came from someone else. The drill came from refusing to stop at the grill.

Both, in generic form with my name and my internals stripped out, are at [github.com/smattymatty/my-skills](https://github.com/smattymatty/my-skills). They're shaped for my head, not yours. Read them, disagree, and build the version that fits.
