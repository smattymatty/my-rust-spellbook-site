---
title: "Grill and Drill with Skill"
published_at: 2026-06-30
carved: true
kind: post
description: "A skill is somebody else's judgment running in your assistant, and it's just markdown. I took Matt Pocock's grill-me, bent it to fit my head, and built the inverse skill, drill, that the original never reached. Read your skills, then make them match your workflow."
quote: The drill came from refusing to stop at the grill.
tags:
  - skills
  - ai-assisted-engineering
  - grill
  - drill
  - owning-your-tools
---

Every skill you add to your AI assistant is a small transfer of judgment. Somebody decided how a piece of thinking should go, what gets asked, in what order, when to stop, and wrote it down so your assistant runs it instead of improvising. Read it before you run it and every one of those decisions is yours to keep or change.

The good news is that the lock on that door is barely a lock at all. A skill is just markdown, and the thinking inside it is just words. There's no compile step, no API to learn, no version resolver deciding what you got, none of the friction that keeps you from cracking open a package off npm or pip and changing it. You open the file, you read it, you rewrite the parts that were written for someone else's head. The skill you want is one read away from the skill the author shipped.

## Read it first

Running the install command and never opening the file is the same reflex that has us all running dependencies we've never read, and most of the time that reflex is fine. A library does a mechanical job the same way in everyone's hands. You give it bytes, it sorts them, and it doesn't matter whose attention span the author had.

A skill is the case where it does matter. The author tuned it to their attention span, their team, their codebase, their sense of a good question. Read it and you get the useful part on its own terms, and you see why it's shaped the way it is, which is the question that turns a skill you use into one you own.

## Grill, and what I changed about it

The skill I started from is Matt Pocock's [grill-me](https://www.aihero.dev/skills-grill-me). The idea behind it is good. Instead of asking the assistant to plan something for you, you make it interview you, one question at a time, walking down every branch of the decision tree until the two of you share an understanding of what you're building. He later extended it into [grill-with-docs](https://www.aihero.dev/grill-with-docs), which reads a glossary and a set of decision records as it goes, sharpens fuzzy terms against them, and writes the resolutions back down so the next session starts further ahead. I use the bones of both every week.

I changed three things, and none of them are cosmetic.

The first is multiple choice over open prose. When the answer to a question is a choice between two or three named options, the skill hands me buttons instead of a blank. Picking the recommended option costs me nothing, so I can accept by default and keep moving. Typing a paragraph costs energy I'd rather spend on the questions that need a paragraph.

The second is a progress bar on every reply. A filled-and-empty bar, a count of questions left, a tally of docs touched. It tracks how close the design tree is to resolved, not how many questions I've answered. It sounds like a toy. For the kind of brain that loses the thread of a long session, the visible pile growing is what keeps me in the chair.

The third, and the one the other two serve, is that many small questions beat a few big ones. A twelve-part question with sub-bullets is a wall I bounce off. Five one-line questions in a row is a rhythm I can stay inside. So the skill is forbidden from stacking two questions in a turn, and it's told to keep each one to a sentence or two and save the reasoning for after I've answered.

They're grill bent to fit me. Someone with a different attention span would bend it somewhere else, or not at all.

![A grill session in progress: a progress bar reading 38% with about four questions left and three new docs touched, above a single multiple-choice question about a sim/render boundary with three named options and a Rust/JS code snippet.](/media/images/blog/inline/grill-example.png)

## Drill, the part that wasn't there

Here's where adapting turned into building.

The longer I used grill, the more I noticed it only ran in one direction. Grill pulls decisions out of your head and writes them down, into a glossary or a decision record. It's generative. It produces the canon. But once the canon exists, nothing in the toolkit checks whether it ever made it back into the one place that matters, your head, where a customer's question lands.

Reading a doc doesn't put it there. Take an architectural invariant you set yourself, some rule about what's allowed to cross a boundary. You wrote it. You'd know it on the page in a heartbeat and nod along. That's recognition, and it's cheap. Recall is producing it cold, with its edges and the failure it's guarding against, when someone asks. The two feel like the same knowledge right up until someone makes you do the second one, and only one of them holds when the system is on fire.

So I built the inverse skill. Drill takes what's already written down and tests whether a head holds it, by making that head solve tight, constrained problems with answers derivable straight from the source. A concrete problem and "what's the answer," with no open-ended discussion of the tradeoffs. The struggle is the whole point. If the skill explained the answer before I attempted it, I'd parrot it back and learn nothing.

The piece I'm proudest of is what happens when my answer and the doc disagree. The easy design assumes the answer key is right and marks me wrong. Drill refuses to. A divergence is ambiguous on purpose, because maybe my mental model drifted, or maybe the doc went stale while the system moved on under it. The skill forces that fork into the open and makes me call it, my error to relearn or a stale doc to fix. A quiz that always trusts its own key can't catch a rotten key. Mine has to ask.

And the fork has teeth on exactly this kind of rule. Fumble the edges of an invariant you designed and the drill makes you call it. Did your own grasp of it drift, or did the system move and leave the doc describing a boundary that no longer exists? One answer sends you back to relearn the rule. The other sends you to fix a doc that's quietly lying about how your own system is wired, and a doc that lies about a security boundary is worse than no doc at all. Either way you found it at a desk instead of in a postmortem. Grill wrote the invariant down months ago. Drill is the thing that checks whether writing it down was ever enough to hold it. Usually it isn't.

The two run as a loop. Grill at the front of a piece of work to decide and record, drill at the back to verify the record landed. The docs are the shared ground between them. One of them did not exist until I stopped treating the other as finished.

## What rebuilding gave me

Drill exists because I rebuilt grill instead of installing it. The whole second skill is the residue of asking, over and over, why is this mechanism here and what does it not cover, and you only ask that when you're rebuilding a thing.

Pocock's idea is the seed of everything here and I'll say so anywhere. What reading his skill bought me was seeing the questions he'd answered, so I knew where his constraints ended and mine began, and I could find the gap his version didn't reach. Taking the thing apart to see why it's shaped the way it is was the step that paid.

I build a company on the premise that you should own the layers you depend on rather than rent them from someone who can change them under you on their own schedule. A skill you've read and bent is that, one layer up. Your own reasoning process is the layer most worth owning.

So take the parts that are good. I took the Socratic spine and the write-it-down discipline, and I'm glad I did. Then make them yours, bend them to your own head, and keep going past the point where the original stopped. The grill came from someone else. The drill came from refusing to stop at the grill.

Both, in generic form with my name and my internals stripped out, are at [github.com/smattymatty/my-skills](https://github.com/smattymatty/my-skills). They're shaped for my head, not yours. Read them, disagree, and build the version that fits.
