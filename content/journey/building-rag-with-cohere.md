---
title: "Building RAG Search for My Docs, Powered by Cohere"
published_at: 2026-07-13
carved: false
kind: post
featured: false
description: "Semantic search for the Storm Developments guide: Cohere Embed and Rerank over pgvector, the bugs I hit building it, and the cost controls that came out of them."
quote: Embedding is billed work, and the index should never pay for the same sentence twice.
tags:
  - rag
  - cohere
  - pgvector
  - search
  - storm-buckets
  - canadian-digital-sovereignty
---

Storm Developments has a guide - setup walkthroughs for Storm Buckets, custom domains, site hosting, everything I ship. It also had no search. Thirteen pages is small enough to browse and just big enough that "where did I explain path-style addressing" starts costing real minutes, and a customer with a question in their head shouldn't have to guess which page I filed the answer under.

So I built semantic search for it. Type a question in natural language, get the five most relevant sections back, click one, land on the heading that answers it. It runs on Cohere's Embed and Rerank models over pgvector, in the Postgres I already operate. There's no LLM in the pipeline. Nothing generates text, so nothing can make an answer up - it finds what I wrote and ranks it.

## Why Cohere

The honest first reason: Cohere is a Canadian AI company, headquartered in Toronto, and if you've read anything else on this blog you know that matters to me. When the tool I need exists from a Canadian company, that's where I start.

The second reason is that their product split matches the problem exactly. Doc search doesn't need a chatbot. It needs two narrow things done well: turn text into vectors that capture meaning (Embed), and given a query and a pile of candidates, rank the candidates by how well they actually answer it (Rerank). Those are Cohere's specialties, sold as plain APIs. I wanted retrieval, not generation, and I got to buy exactly retrieval.

## How it works

The pipeline has a write side and a read side, and they're two small files.

**The write side** is a management command, `reindex_docs`. It walks every page on the public guide surface - the same visibility filter the sitemap uses, so internal docs never enter the search index - and splits each page's markdown on `##` headings. Each chunk gets embedded with `embed-v4.0` at 1024 dimensions and stored in a `DocEmbedding` row with its slug, chunk index, heading, and a content hash.

The content hash is the detail I'd keep in any redesign: on every run, a chunk whose text hasn't changed is skipped entirely. No re-embed, no re-pay. Reindexing after editing one page costs one page's worth of API calls, not the whole corpus. Embedding is billed work, and the index should never pay for the same sentence twice.

**The read side** is a two-step. Embedding similarity alone ranks "roughly about the same topic" above "actually answers this question" - that gap is the whole reason rerank models exist. So a query gets embedded, pgvector's cosine distance narrows the corpus to a 24-chunk candidate pool (cheap, local, no API call), and then `rerank-v4.0-fast` scores that pool against the raw query text and returns the top five. The fast rerank variant, not the pro one: this runs synchronously inside a search request, so latency beats the last increment of ranking quality.

First real test after indexing: searching "certificate provisioning" put the exact section I'd recently patched at the top with a 0.79 relevance score, and the next result at 0.33. That gap is the rerank layer doing real work - cosine distance alone had those much closer together.

Each result also carries an anchor, computed with the same heading-slug function that stamps `id` attributes onto every rendered heading. Click a result and you land on the section, not the top of the page. More on why that exists in a moment.

## What broke, in order

The pipeline took a weekend. Most of what I know about running this now came from the bugs after it.

**Typing one word cost five API calls.** The search box debounced at 250ms with a 2-character minimum - reasonable numbers for autocomplete against your own database, and wrong numbers when every settled pause is a billed embed-plus-rerank round trip. Typing "certificate" at a normal human pace fired separate paid searches for the fragments along the way. I caught it on Cohere's billing dashboard at seven whole cents and fixed it before it could become dollars: 450ms debounce, 3-character floor, an AbortController so a superseded request is actually killed, and - the real fix - a query cache.

The cache is the real fix. It's keyed on the normalized query text plus the index's latest `updated_at` timestamp, which means any reindex that touches even one chunk changes every key. Stale results can't outlive the content they came from, and there's no invalidation step for anyone to forget. A repeat query went from 5.7 seconds of real API round trip to 20 milliseconds. And since every result is public docs content, the cache is global: one visitor's search warms it for everyone.

**Raw markdown leaked into the results.** Snippets are previews, not rendered pages, so customers were seeing literal `**DNS verified**` asterisks, raw table rows, and the opening syntax of my custom SpellBlocks. The stripper now handles all of it. A purist would render the markdown properly and strip the HTML. I wrote regexes instead - a stray list marker in a snippet doesn't hurt anyone, and the renderer route would couple search to the whole template stack for a preview string.

**Five results, one page, and every click landed at the top.** Searching "certs" correctly surfaced five different sections of the custom-domains page - and clicking any of them dumped you at the top of the page, five indistinguishable times. The ranking was right; the navigation threw the precision away. That's where the anchor field came from: the search now reuses the exact slug algorithm the renderer uses for heading ids, so a result click lands on the section that matched.

**"No matches" flickered on every keystroke.** The empty state rendered whenever there were no results and nothing in flight - which, thanks to the debounce delay, was true for a quarter second after every character. The fix was tracking "this query was actually searched and found nothing" as its own piece of state instead of inferring it from "not currently loading." Now "No matches" only shows once a search for the current text has really finished.

## The controls that stay

Beyond the fixes, some guardrails are permanent. The endpoint is rate-limited per IP. Dev environments default the whole Cohere integration off - a kill switch in settings - so an API key sitting in a `.env` file can't spend money by accident; turning it on for a testing session is one explicit line. That one exists because I know exactly who is most likely to run the indexer against the wrong environment at midnight.

## What it adds up to

Thirteen pages, 83 chunks, one `make reindex`, and a search box on every guide page that understands questions instead of keywords. The whole backend is about 150 lines split across two readable files, on the Postgres I already run, with the two AI calls in the pipeline doing the two things AI is actually good at here - meaning and ranking - and nothing else.

And the whole thing runs on a Canadian AI company's models, over a database I operate, serving docs about infrastructure hosted in Canada. That lined up nicely.

{~ card ~}
The [Storm Developments guide](https://stormdevelopments.ca/guide/) is where this search box lives - docs for Storm Buckets and everything else I build, all hosted entirely in Canada. Go ask it something.
{~~}

Own your tools. Watch your bills.
