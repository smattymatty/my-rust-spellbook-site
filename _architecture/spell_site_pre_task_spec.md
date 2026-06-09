# Pre-Task Spec: Spell Site - Generator / Theme / Extension System

**Status:** Pre-task, post-grilling (revision 1). Ready for implementation planning *after* the user reads this cold and marks corrections.

**Working directory at time of writing:** `rust-spellbook-static` (the Rust engine that becomes Spell Site's runtime; see §3.9).

---

## 0. Delta from v0

This revision rewrites the pre-task spec the user handed in mid-session. The original document remains the source of intent; this one captures what changed under grilling and what the new commitments are. The deltas are non-trivial:

- **§3.2 rewritten.** Generation is server-side on a Storm-managed VPS, not in the browser. The browser holds the bucket key; generated files pass-through-stream back to the browser, which PUTs them to the user's bucket.
- **§2.3 rewritten.** Extensions are WASM modules (`.wasm`), not JS. Run on the VPS via wasmtime, capability-gated. Authored in Rust + cargo-component + WIT/Component Model.
- **§2.4 rewritten.** Core runtime is the Rust engine in `rust-spellbook-static`, not TypeScript in the browser. Same engine runs in two modes: CLI for the personal site, server for Spell Site.
- **§3.4 stretched.** Layout-is-structural still holds in spirit, but per-generator "feature toggles" in config (e.g., vault's sidebar/backlinks/graph) are policy, not new generators.
- **§3.8 replaced.** Form-shaped abstraction earned by the resume generator; collection abstraction earned by the Obsidian-vault generator in the same v1 cut. Both ship together.
- **§5 expanded.** v1 = Resume *plus* Obsidian-vault (collection-shaped). Two Storm-built WASM extensions: obsidian-features and commit-fetcher.
- **§4.5 resolved.** Template engine is **Tera**, not Mustache. Mustache was a v0 carry-over from when generation ran in the browser; under server-side Rust + feature-toggle conditionals + collection iteration, Mustache fights the design at every turn. Tera is well-trodden in SSGs specifically (Zola precedent).
- **New §3.9, 3.10, 3.11** capture commitments that the v0 spec didn't have but that the grilling produced (engine reuse, sovereignty under server-side generation, single-tenant v1).
- **New §4.9, 4.10, 4.11** capture residual open items.
- **Deferred:** extension marketplace contribution model (§4.9). Most other v0 open questions resolved.

---

## 1. The thesis

Storm Buckets evolves from "managed S3" into a platform for sovereignty-conscious creators who want a personal site, resume, vault publication, or game host on Canadian infrastructure with open source underneath. Spell Site is the layer that helps them produce the actual site files.

**Product framing:** "cast your site like a spell." A user fills out a form (or points at their vault), picks a theme, optionally adds extensions, hits a button, and a real static site appears in their Buckets bucket.

**Engine framing:** Spell Site is a *front door* on the same Rust engine that powers the user's personal site CLI (`rust-spellbook-static`). One engine, two modes. See §3.9.

---

## 2. The four data types

Three are declarative (data). One is a code runtime. Composition is via well-defined slots.

### 2.1 Generator (declarative)

A generator declares *how a site is composed* - what config it accepts, what singleton files it produces, what collections it iterates over, what tokens themes can override, and what extension slots are available.

Fields:

- **Identity:** `id`, `name`, `version`, `author`, `description`
- **`config_schema`:** JSON Schema for the inputs the generator needs from the user. May include a `features` sub-object for in-generator layout toggles (per §3.4).
- **`tokens`:** the CSS custom-property vocabulary this generator exposes for themes to override. Listed by category (color, font, size, space, radius, shadow, motion).
- **`singletons`:** map of one-off output files (e.g., `index.html`, `404.html`, `style.css`) to their Tera template paths.
- **`collections`:** map of named collections to their declarative configuration (source glob, per-item template, item URL pattern, frontmatter contract, optional index template). Empty for form-shaped generators.
- **`assets`:** static files shipped with the generator.
- **`extensions`:** explicit dependencies on named WASM extensions (e.g., `storm.obsidian-features.v1`).
- **`extension_slots`:** named hook points where the generator allows extensions to attach, each with a trigger phase (`before_render`, `render`, `after_render`).

#### Example: form-shaped generator (Resume)

```json
{
  "schema_version": 1,
  "id": "storm.resume.v1",
  "name": "Resume",
  "version": "1.0.0",
  "config_schema": {
    "type": "object",
    "required": ["name", "tagline"],
    "properties": {
      "name":            { "type": "string" },
      "tagline":         { "type": "string" },
      "github_username": { "type": "string" },
      "forgejo_user":    { "type": "string", "format": "uri" },
      "show_commits":    { "type": "boolean", "default": true }
    }
  },
  "tokens": {
    "color":  ["primary", "accent", "surface", "text", "text-muted"],
    "font":   ["heading", "body", "mono"],
    "size":   ["scale-1", "scale-2", "scale-3"],
    "space":  ["1", "2", "3", "4", "5"],
    "radius": ["sm", "md", "lg"]
  },
  "singletons": {
    "index.html": "templates/index.html.tera",
    "404.html":   "templates/404.html.tera",
    "style.css":  "templates/style.css.tera"
  },
  "collections": {},
  "assets": ["assets/icons.svg"],
  "extensions": ["storm.commit-fetcher.v1"],
  "extension_slots": [
    { "name": "commit_fetcher", "trigger": "before_render" },
    { "name": "head_meta",      "trigger": "render" }
  ]
}
```

#### Example: collection-shaped generator (Obsidian-vault)

```json
{
  "schema_version": 1,
  "id": "storm.obsidian-vault.v1",
  "name": "Obsidian Vault",
  "version": "1.0.0",
  "config_schema": {
    "type": "object",
    "required": ["site_title"],
    "properties": {
      "site_title":  { "type": "string" },
      "vault_root":  { "type": "string", "default": "content/", "description": "Path within the user's bucket where vault notes live" },
      "features": {
        "type": "object",
        "properties": {
          "sidebar":   { "type": "boolean", "default": true },
          "backlinks": { "type": "boolean", "default": true },
          "graph":     { "type": "boolean", "default": false }
        }
      }
    }
  },
  "tokens": {
    "color":  ["primary", "accent", "surface", "text", "text-muted", "link", "link-visited"],
    "font":   ["heading", "body", "mono"],
    "size":   ["scale-1", "scale-2", "scale-3"],
    "space":  ["1", "2", "3", "4", "5"],
    "radius": ["sm", "md", "lg"]
  },
  "singletons": {
    "index.html": "templates/index.html.tera",
    "404.html":   "templates/404.html.tera",
    "style.css":  "templates/style.css.tera"
  },
  "collections": {
    "notes": {
      "source":              "{vault_root}/**/*.md",
      "item_template":       "templates/note.html.tera",
      "item_path":           "/{path_no_ext}/",
      "frontmatter_required": ["title"]
    }
  },
  "assets": ["assets/icons.svg"],
  "extensions": ["storm.obsidian-features.v1"],
  "extension_slots": [
    { "name": "markdown_transform", "trigger": "before_render" },
    { "name": "backlink_resolver",  "trigger": "render" },
    { "name": "head_meta",          "trigger": "render" }
  ]
}
```

**Collection DSL semantics (sketch - see §4.10 for residual sub-decisions):**

- `source` is a glob, with `{config_var}` interpolation from `config_schema` values.
- `item_template` is rendered once per matched file. Tera context includes `{ frontmatter, content_html, path, slug, ...config }`.
- `item_path` is the output URL/file path. Supports `{slug}`, `{path_no_ext}`, frontmatter fields, and config values via Tera-style interpolation.
- `frontmatter_required` is validated at gen time; missing fields fail noisily with the offending file path.
- Optional: `index_template` and `index_path` for aggregate pages; not used in the v1 vault generator (the singleton `index.html` plus extension-provided nav covers it).

### 2.2 Theme (declarative)

A theme is **pure CSS token overrides** plus optional font assets. Themes never modify HTML structure or ship JS. If a theme needs JS behavior, that's an extension, not a theme. (See §3.3 and §3.4.)

Fields unchanged from v0:

- **Identity:** `id`, `name`, `version`, `author`
- **`for_generator`:** which generator(s) this theme targets
- **`tokens`:** flat dotted-path map of token → CSS value
- **`font_assets`:** optional list of font URLs
- **Preview metadata:** thumbnail, description

### 2.3 Extension (WASM module + declarative manifest)

Extensions are where executable behavior lives. WASM modules executed on the VPS by wasmtime, capability-gated by the host. They attach to named slots on a generator.

Fields:

- **Identity:** `id`, `name`, `version`, `author`
- **`hooks_into`:** list of extension slot names (must match slots declared on the consuming generator)
- **`code`:** path to the compiled `.wasm` artifact
- **`wit_world`:** the WIT world the module implements (e.g., `obsidian-features:0.1`). The engine validates that the loaded module matches the declared world.
- **`permissions`:** declared capabilities (`fetch_external`, `read_collection_content`, `read_bucket_metadata`, ...). The wasmtime host bridge wires the corresponding host functions only if the permission is declared; the user is shown declared permissions at install time.
- **`config_schema`:** extensions can take their own config, validated like generator config.

Authoring stack (per §4.9 resolution):

- Rust + cargo-component, targeting `wasm32-wasi-preview2`.
- Host/extension interfaces defined in WIT files (live in the engine repo). `wit-bindgen` generates bindings for both sides.
- The engine is the only thing that ships first-party extensions in v1.

Example:

```json
{
  "schema_version": 1,
  "id": "storm.obsidian-features.v1",
  "name": "Obsidian features (backlinks, transclusion, file-tree nav)",
  "version": "0.1.0",
  "hooks_into": ["markdown_transform", "backlink_resolver"],
  "code": "extension.wasm",
  "wit_world": "obsidian-features:0.1",
  "permissions": ["read_collection_content"],
  "config_schema": {
    "type": "object",
    "properties": {
      "transclusion_syntax": { "type": "string", "enum": ["obsidian", "off"], "default": "obsidian" }
    }
  }
}
```

### 2.4 Core runtime (code)

**The Rust engine in `rust-spellbook-static`.** Two operating modes (per §3.9):

- **CLI mode** - used by the user (Mr. Storm) to generate his own personal site from content + data + a hardcoded generator + Tera templates. This is the existing `rust-spellbook-static` binary, adapted to load generators from manifests.
- **Server mode** - used by the Spell Site flow. The engine runs as a long-lived process on a dedicated VPS managed by Storm Pulse. It accepts generation requests from the Storm backend, runs the requested generator with the user's config + content, and streams the generated files back through the backend to the user's browser.

The engine's responsibilities in order (per generation request, in both modes):

1. Load a generator manifest + its templates and assets.
2. Validate the user's config against the generator's `config_schema`.
3. Load the chosen theme manifest; merge its token values over the generator's token defaults.
4. Load declared extensions; instantiate them in wasmtime instances with host functions wired per declared permissions.
5. For each singleton: render the Tera template with context `{ config, tokens, features, ...assets }`, firing extension hooks at declared slots.
6. For each collection: walk the source glob, parse each item's frontmatter, render `item_template` once per item, fire per-item hooks at declared slots, write to `item_path`.
7. Assemble all output files into a coherent set.
8. In CLI mode: write to `dist/`. In server mode: stream files (one at a time, or chunked) through the WebSocket back to the browser.

The engine never executes generator code, because generators are data (§3.1). It does execute extension code, with wasmtime isolation + capability gating (§3.5, §4.11).

---

## 3. Architectural commitments

These are the load-bearing decisions. The agent should not re-litigate them.

### 3.1 Generators are data, not code

User-contributed and AI-generated generators cannot be arbitrary code without serious sandboxing. Declarative manifests sidestep that problem entirely - the runtime is the only code, and it ships with Storm.

Under WASM extensions (§2.3), this commitment is *strengthened*: even the executable code path is locked into a capability-gated sandbox. There is no path by which a user-contributed generator can execute arbitrary native code on Storm infrastructure.

### 3.2 Generation runs server-side on a Storm-managed VPS (rewritten from v0)

Generation runs on a dedicated VPS, orchestrated by Storm Pulse. The Rust engine performs all work. The browser holds the bucket key throughout; generated files stream pass-through from the VPS through the Storm backend to the browser, which then PUTs to the user's bucket.

**Sovereignty story (narrowed):**

- Storm *never* holds your bucket key.
- Storm *never* writes to your bucket.
- Storm *never* persists your generated files on disk.
- Storm *does* see your generation config + your content + your generated output during the pass-through (this is unavoidable; generation has to happen somewhere).

This is a real narrowing from v0's "browser-only generation" framing. The previous framing was stronger on optics but worse on every practical axis (browser perf, JS toolchain pain, code reuse with the user's existing Rust engine, AI extensibility, sandbox quality). The current framing is honest about what Storm sees and crisp about what Storm does not.

### 3.3 Themes are pure CSS

Themes override CSS tokens. They never modify HTML structure or ship JS. If a theme needs JS behavior (animations, interactivity), that's an extension. The line is firm.

### 3.4 Layout is structural, not theming - with feature-toggle exception (revised from v0)

Single-column vs two-column resume = different generators. Themes are visual treatment over a fixed structure.

**Exception:** generator-declared feature toggles in `config_schema` (e.g., the vault's `features.sidebar`, `features.backlinks`, `features.graph`) are *user policy*, not theme. They live in the generator's config because they're structural variants the generator author has explicitly designed for. They are not theme territory and they are not separate generators.

### 3.5 Extension slots are explicit

Generators declare which slots exist. Extensions declare which slot(s) they hook into via `hooks_into`. The match is validated at gen-start time. Extensions don't get to inject arbitrary code anywhere - they get the slots the generator chose to expose, and they get only the host capabilities they declared and the user approved.

### 3.6 Config travels with the bucket

The user's filled-out config is persisted as `.storm/site.json` in their own bucket. The dashboard reads it on load (pre-fills the form), writes it on regenerate. Implications:

- The user owns their config.
- Theme switches don't require re-filling the form.
- Hand-editing the JSON is supported.
- Migrating between buckets means copying the file.

### 3.7 Token vocabulary is extracted, not designed upfront

Build the resume generator with literal CSS values first. Refactor to tokens when building the second theme. Don't try to enumerate the "complete" token list before any themes exist.

### 3.8 Ship one form-shaped generator and one collection-shaped generator together; both earn their abstractions (replaces v0 §3.8)

v0 said "ship one generator before generalizing." That principle was right for its scope (form-shaped generators only). Under the revised v1, the *form* abstraction and the *collection* abstraction are both load-bearing; each must be exercised by a real generator at v1 launch, not designed in isolation:

- The Resume generator validates the form-shaped abstraction (config-driven, singletons-only).
- The Obsidian-vault generator validates the collection-shaped abstraction (source globs, per-item rendering, frontmatter contracts).

No abstraction beyond what these two exercise gets to ship in v1. Pagination, taxonomies, multi-collection-per-generator, derived index pages with computed aggregates: all out unless they're forced by Resume or Vault.

### 3.9 Engine reuse (new)

`rust-spellbook-static` *is* Spell Site's runtime. The Rust engine has two operating modes (CLI and server, per §2.4). There is one codebase, one template engine (Tera, per §4.5), one extension toolchain (cargo-component + WIT, per §4.9), one schema for generators/themes/extensions.

Cost paid: Mr. Storm's existing askama-based templates in `rust-spellbook-static` are rewritten in Tera. Cheap (~3 templates, half a day).

Benefit purchased: one engine to maintain forever, no schema drift between personal-site and customer-site stacks, every improvement to one benefits the other.

### 3.10 File round-trip preserves sovereignty (new)

Per §3.2, but stated as a standing commitment: generated files round-trip through the user's browser. The Storm backend acts as a network broker (WebSocket relay); it does not buffer to disk, does not cache, does not log file contents. The browser receives files as they're generated and PUTs to the user's bucket using the in-memory key.

UX implication: generation is synchronous from the user's perspective. The dashboard shows a "generating..." state; closing the browser mid-gen cancels the gen and requires re-triggering. Async resumable generation is a v2 problem.

### 3.11 v1 is single-tenant per generation (new)

One VPS, serialized queue. One generation runs at a time across all Buckets customers. Latency = your gen time + queue wait. This is acceptable for v1 launch scale; multi-VPS fleet management and per-tenant isolation are v2 problems when load justifies them.

The wasmtime sandbox configuration assumes single-tenant: extensions only need to be isolated from the *host engine*, not from other tenants. Per-extension fuel limits and memory ceilings still apply (a runaway extension shouldn't hang the engine), but tenant-from-tenant isolation is a non-problem at v1 scale.

---

## 4. Open questions (residual)

Most v0 open questions resolved during grilling. The residuals:

### 4.1 - Resolved
Sandbox mechanism resolved as wasmtime (see 4.11 for remaining config details).

### 4.2 - Resolved
Multi-forge data fetching: server-side, so CORS is not a constraint. Lives in `storm.commit-fetcher.v1` WASM extension with declared `fetch_external` capability. Host bridges that capability to `reqwest` or similar.

### 4.3 Browser key storage (downgraded urgency)

Under server-side generation, the bucket key is used only at the end of a generation cycle (for the final round of PUTs), not continuously during gen. This reduces - though does not eliminate - XSS exposure.

**v1:** in-memory only (lost on reload, re-enter on next session). Acceptable because key entry is a one-time per-session ceremony, not a continuous-use credential.

**v2:** Service-Worker-backed cross-tab persistence with explicit user opt-in and a clearly-communicated security model.

### 4.4 Where Storm-built generators ship

**v1:** bundled in the dashboard frontend, referenced by built-in IDs.

**v2 (marketplace):** generators ship from a public Buckets bucket the dashboard fetches at runtime, with versioning via the manifest's `version` field and a Storm-signed integrity manifest.

### 4.5 - Resolved
Template engine: **Tera**. Pure Rust, Django/Jinja2-style, well-trodden in SSGs specifically (Zola precedent), supports conditionals + loops + inheritance + macros + filters which are all needed for the feature-toggle templates and collection iteration.

### 4.6 Manifest format

**JSON** for generators, themes, extensions. AI-friendly, ubiquitous, no parser drama. (TOML for user-facing config files like `.storm/site.json` stays as v0 said - `.json` for site.json was already implicit; confirm during implementation.)

### 4.7 Generator validation at upload time (still deferred, depends on 4.9)

When user-contributed generators land (post-v1, post-marketplace-policy-decision), validation needs to address:

- JSON Schema validation of the manifest itself
- Tera template safety (no template injection escape via `safe` filter abuse, etc.)
- Asset content-type checking
- WASM-extension-dependency resolution (cannot depend on extensions outside the marketplace)

All deferred until §4.9 lands.

### 4.8 - Resolved
The Obsidian-vault generator example in §2.1 illustrates how the collection abstraction handles vault-shaped sites. Backlinks, transclusion, and file-tree navigation live in the `storm.obsidian-features.v1` WASM extension hooked into `markdown_transform` (rewrite `[[note]]` syntax to resolvable links during markdown parse) and `backlink_resolver` (compute backlinks per-note at render time using accumulated graph state).

### 4.9 Extension marketplace contribution model (deferred - explicit)

For v1: Storm-curated only. Storm builds and ships extensions; users compose them into generators but cannot submit their own.

For v2+: open marketplace? Capability-restricted auto-approval? Storm review queue? Decision deferred. Note the half-life: every v1 authoring-tool / docs / extension-SDK decision biases the eventual answer.

### 4.10 Collection DSL sub-details (residual)

The DSL shape in §2.1 covers the common case but leaves details unspecified:

- **URL pattern syntax:** Tera-style interpolation (`{slug}`, `{path_no_ext}`, frontmatter fields). Confirm during implementation.
- **Frontmatter format:** YAML (Obsidian convention; gray_matter already supports it; matches existing `rust-spellbook-static` content). Validate against `frontmatter_required`, fail noisily on missing fields.
- **Multi-collection per generator:** Supported in the schema (`collections` is a map keyed by name), not exercised by either v1 generator. v1 vault uses a single `notes` collection; multi-collection contracts firm up when the second collection-shaped generator forces them.
- **Sort / filter / group / aggregate index pages:** out of scope for v1 unless the vault generator demands them (it doesn't, at the minimum-viable cut: no chronological sort, no tag pages, no aggregate filters).

### 4.11 Wasmtime sandbox defaults (residual)

Settled at implementation time, but the shape:

- **Engine:** wasmtime ~22+, Component Model enabled, WASI preview2.
- **Per-call fuel limit:** enforced; precise value tuned during implementation.
- **Memory ceiling:** ~64MB per extension instance initial; tune from real measurements.
- **Filesystem access:** none; extensions access content only via host functions.
- **Network access:** none by default; granted via `fetch_external` permission, bridged through a host-provided `fetch` function with a request allow-list (or at least logging).
- **Threading:** single-threaded per instance.
- **Time limit:** wall-clock cap per call (~30s initial); abort and report.

---

## 5. What v1 actually looks like

Strip everything aspirational, the actual first shippable feature is:

- **Two generators:**
  - **Resume** (form-shaped, no collections)
  - **Obsidian-vault** (collection-shaped, exercises the `notes` collection)
- **Themes:** at least 2 per generator (a light default and a dark Midnight-style), possibly shared across generators if token vocabulary overlaps.
- **Two Storm-built WASM extensions:**
  - **`storm.commit-fetcher.v1`** - fetches commit data from Forgejo / GitHub / GitLab / Codeberg / Sourcehut. Declares `fetch_external`. Used by Resume.
  - **`storm.obsidian-features.v1`** - backlinks, `[[transclusion]]`, file-tree-nav. Declares `read_collection_content`. Used by Obsidian-vault.
- **The Rust engine** in `rust-spellbook-static`, refactored to load generators from manifests, with Tera replacing askama and wasmtime embedded. Two modes: CLI (used for the user's personal site) and server (used for Spell Site).
- **A new dashboard route:** `Bucket → Spell Site` panel.
- **A generation VPS** spun up and Pulse-managed, running the engine in server mode.
- **Pass-through file streaming** from VPS → backend (broker) → browser → user's bucket.
- **A serialized generation queue** in the backend; one gen at a time.

That's the v1 cut. Everything else - marketplace, AI-generated generators, custom domains, fleet generation, async resumable gen, multi-collection generators, taxonomies, pagination, the `spell.site` domain - is downstream of v1 working end-to-end.

---

## 6. Out of scope for v1

- User-contributed generators or themes (deferred per §4.9).
- AI-generated generators (deferred; revisit post-marketplace).
- A marketplace UI.
- `spell.site` domain (depends on custom-domain feature, its own multi-week project).
- Live previews during generation (just regenerate to see changes).
- WASM game hosting as a generator type (the platform supports it via static file upload; doesn't need a generator).
- Theme switching without regeneration (v2 maybe, if themes really are CSS-only - but v1 just regenerates).
- Multi-tenant concurrent generation (v2; v1 is single-tenant serialized per §3.11).
- VPS fleet management (v2; v1 is one VPS).
- Async resumable generation (v2; v1 is sync per §3.10).
- Multi-collection-per-generator (schema supports it, no v1 generator exercises it per §4.10).
- Taxonomies, tag pages, RSS feeds, pagination (none of v1's two generators need them; earn them with a third generator).
- Browser-side cross-tab key persistence (Service Worker; v2 per §4.3).

---

## Appendix: What the grilling decided, in order

For future-you / future-contributor to reconstruct how the spec arrived here:

1. **Scope of grilling:** "the why" (purpose & characteristics), not seam audit or governance.
2. **Engine audience identity:** "both" - personal NOW, general LATER, designed for that path. Initially flagged as the hardest of four options; vindicated by the eventual engine-reuse architecture (§3.9).
3. **General-purpose audience:** Storm Buckets customers specifically, not Rust-SSG-niche or open-market SSG users.
4. **Template seam (early, pre-reframe):** orphaned by the spec reframe; revisited as §4.5 resolution = Tera.
5. **Project relationship:** server-side Rust on dedicated VPS, Pulse-managed (the user's invented fifth option, off my multi-choice list).
6. **Proceed past architecture shift:** keep grilling, don't pause.
7. **Extension runtime:** WASM via wasmtime, capability-gated.
8. **Marketplace contribution model:** deferred (see §4.9).
9. **Collection abstraction in v1:** yes, extend §2.1 now (overriding v0 §3.8).
10. **§3.8 override:** conscious, with the understanding that the abstraction earns its place via a second generator.
11. **v1 contents:** Resume + a second collection-shaped generator.
12. **Second generator:** Obsidian-vault (most differentiated, ties to sovereignty narrative).
13. **Collection DSL kind:** declarative (data, not pipeline-of-operations).
14. **Where Obsidian features live:** Storm-shipped WASM extension (`storm.obsidian-features.v1`), reusable by other graph-shaped generators in the future.
15. **WASM authoring stack:** Rust + WIT/Component Model + cargo-component + wasmtime ~22+.
16. **File round-trip flow:** pure pass-through stream; Storm holds nothing on disk.
17. **Multitenancy:** one VPS, serialized queue; v2 problem when load demands more.
18. **Vault layout variants:** feature toggles in generator config (one generator, multiple structural variants via `features: {...}`).
19. **Template engine:** Tera, after a frustrated-but-correct user push-back on the carried-forward Mustache assumption.

Pattern noted across the session: the user picked the higher-surface / less-recommended option in 6 of the first 8 forks, then converged toward the recommended option in 5 of the last 6. The architecture is consequently more capable than v0's "ship one generator" cut but more pragmatic than the maximal "design everything now" trajectory the early forks were heading toward.
