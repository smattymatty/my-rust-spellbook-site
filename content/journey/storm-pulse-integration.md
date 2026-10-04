---
title: "Storm Pulse: Integrations"
published_at: 2026-07-03
carved: false
kind: post
featured: false
description: "Storm Pulse manages my S3 nodes and its kernel contains zero S3 knowledge. How a one-person infrastructure company keeps its most privileged code small enough to actually audit."
quote: My management agent has two jobs that pull against each other. It has to be small enough to trust [...] And it has to keep learning new tricks
tags:
  - storm-pulse
  - architecture
  - open-source
  - self-hosting
  - python
---

Storm Pulse is the most privileged program on every server I operate. It runs the commands, reads the state, and holds the local admin credentials for the storage engine underneath [Storm Buckets](https://stormdevelopments.ca/buckets). A compromise of Pulse is a compromise of everything it manages, so its design carries more weight than anything else I've written.

The core of the Pulse agent does not know what Garage is. There is no `if integration.id == "garage"` anywhere in the agent. The kernel that manages my S3 nodes has zero S3 knowledge in it, by design, and writing that design down is how I make sure I still understand it six months from now.

## The problem with privileged software that grows

My management agent has two jobs that pull against each other. It has to be small enough to trust - Pulse is open source so anyone can read the most privileged thing touching their server's services and hold all of it in their head, an audit most commercial fleet agents never offer you. (It's deliberately not root: a rootless, sudo-less user under a sandboxed systemd unit, so even the worst case is bounded.) And it has to keep learning new tricks, because every product I ship needs the agent to drive something new: Garage today, the Caddy edge proxy already, more coming.

If every new capability lands in the core, the second job slowly destroys the first. Each addition means re-reading connect, auth, dispatch, the loops - the security-critical spine - to make sure nothing new leaks into it. As one person, I can't afford an architecture where growth and auditability fight each other. So Pulse is split down that exact line.

## The mental model: a kernel with plugs

The kernel does the universal work, the stuff that's identical no matter what system sits underneath: connect to the dashboard over mTLS, prove identity, verify HMAC-signed commands, run background jobs, ship metrics and logs. Everything that knows how to drive a *specific* system lives in an integration, a small Python package that plugs into the kernel through one contract.

```
                  ┌─────────────────────────────────────┐
                  │              the kernel             │
 dashboard ◀─────▶│  connect · auth · dispatch · jobs   │
 (mTLS/WS)        │  metrics · logs · refresh  · events │
                  └───────────────┬─────────────────────┘
                                  │ iterates whatever registered
            ┌─────────────────────┼─────────────────────┐
            ▼                     ▼                     ▼
      ┌───────────┐         ┌───────────┐        ┌─────────────┐
      │  garage   │         │   caddy   │        │   yours     │
      └───────────┘         └───────────┘        └─────────────┘
```

The contract is a frozen dataclass with three required fields and a menu of optional ones:

```python
@dataclass(frozen=True, slots=True)
class Integration:
    id: str                          # "garage", "caddy", yours
    parse_config: ParseConfig        # raw TOML table -> typed config
    enabled: EnabledPredicate        # is this turned on?
    preconditions: Preconditions | None = None
    specs: BuildSpecs | None = None          # commands it contributes
    discover: CollectState | None = None
    collect_state: CollectState | None = None
    detect: Detector | None = None
    read_affected: ReadAffected | None = None       # post-mutation targeted re-read
    log_enrichers: Mapping[str, BuildLogEnricher] | None = None  # keyed by parser
```

| Field | Required | What it declares |
|-------|----------|------------------|
| `id` | yes | The integration's name. Keys its config table, its slot in the wire payload, and every command it contributes (a command's `group` must equal this, refused at startup otherwise). |
| `parse_config` | yes | Turns the raw TOML table into the integration's own typed config. A bad value soft-disables this integration, never the agent. |
| `enabled` | yes | The on/off predicate, read from that parsed config. |
| `preconditions` | opt-in | Startup health check ("is the container up, is the version supported"). Fails with a named reason the dashboard displays. |
| `specs` | opt-in | The command surface: a builder returning the integration's `CommandSpec`s, job handlers riding on the specs. |
| `discover` | opt-in | One-shot full state read at register time, so the dashboard isn't blind until the first periodic tick. |
| `collect_state` | opt-in | Periodic state collection, riding the metrics-push cadence. Declaring it earns a free `{id}_refresh` command, synthesized by the kernel. |
| `detect` | opt-in | A fast new-resource detector with its own cadence - the one tunable state-read knob, because it bounds a security window. |
| `read_affected` | opt-in | Post-mutation targeted re-read: given a succeeded command's params, which resources changed, re-read only those. The kernel owns the merge and the push. |
| `log_enrichers` | opt-in | Parser name to a builder that turns current state into a log-line enricher (garage stamps `bucket_id` onto its S3 access lines). Keys are disjoint across integrations, fitness-checked. |

Everything past the first three is opt-in. Caddy declares commands and no state loop. A read-only monitor could declare a state loop and no commands. There are no empty stubs to fill in, because a capability you don't have shouldn't cost you boilerplate pretending you do. The last two fields are a day old, and how they got there is the end of this post.

Registration is an import side effect. Your module ends with `register_integration(MY_INTEGRATION)`, and one line in a manifest file imports it:

```python
import stormpulse.garage.integration  # noqa: F401
import stormpulse.caddy.integration   # noqa: F401
```

At startup the kernel loops over whatever registered, checks each one's config, gates it through its preconditions, and merges its commands into the live whitelist. Adding a new integration is a new package plus one import line. Bootstrap doesn't change. Dispatch doesn't change. The auth path doesn't change. The audit surface of the core stays exactly where it was.

Today that import list is the only door: adding your own integration means a fork or an upstream PR. A real plugin loader is deferred on purpose - the most privileged thing on the box shouldn't execute someone else's code until signing and confinement exist, and right now the only integration author is me.

## Illegal states refuse to compile themselves

My favourite piece is how commands are declared. A command is one `CommandSpec` object carrying its schema and, for a long-running job, its handler:

```python
"widget_drain": CommandSpec(
    group="widget",
    command=["widget_drain"],
    timeout=120,
    mode="job",
    handler=lambda params: make_drain_handler(config, params),
),
```

There used to be two structures here - a definition map and a separate name-to-handler map that had to agree. They were kept in sync by a hand-maintained list of expected command names in a test, which is a polite way of saying they were kept in sync by hope. A command and its handler that *can* drift apart *will* drift apart.

Now the dataclass rejects the broken shapes at construction. A job with no handler raises immediately. A subprocess whose binary isn't an absolute path raises immediately. A parameter with neither a validation regex nor a byte cap can't be built at all. The half-registered command, the classic mystery failure, became a `ValueError` on my machine before anything ships. Structural discipline is the only kind that survives me, with my brain off, at midnight.

## The failure model: integrations go dark, the agent stays up

The rule is blunt: a broken integration never takes the agent down. Invalid *core* config - no dashboard URL, no certificate - is fatal, because an agent that can't do its one job should refuse to start. But an integration that fails to parse its config, flunks a precondition, or trips a construction guard just soft-disables. It goes dark, reports a human-readable reason the dashboard displays, and the kernel plus every other integration keeps running.

Garage gets no privilege here that a third-party integration wouldn't.

There is a lighter extension path that needs no fork at all: operators can define their own whitelisted subprocess commands in `stormpulse.toml` - full citizens of the registry, HMAC-verified like everything else, absolute paths and validated params enforced at load. The line between "config can do it" and "you need an integration" is jobs, state collection, soft-disable, and the state-driven extras like post-mutation refresh and log enrichment.

There is exactly one command that breaks the whitelist promise on purpose: `run_verify_block`, which carries opaque shell for my playbook sign-off system. It ships sealed. Unsealing is host-only, makes you type the hostname back, and my dashboard yells at me if the hatch stays open past an hour. That command is a whole post of its own.

The piece I'm building now is the identity gate: wiring the seal into my auth backend so an unseal demands real credentials and the record shows exactly which account opened the hatch, on which host, at what time. Today "who unsealed it" is answered by "me, obviously" - an answer that stops working the day another engineer touches my fleet, or someone else runs Pulse on theirs.

## Why I'm writing this now

The first draft of this post claimed the kernel was garage-free, and a pre-publish audit against the code proved it wasn't: one garage-named module was still living in the agent package. I spent a day moving its two jobs onto the contract - that's the `read_affected` and `log_enrichers` fields above - and now the grep is clean: `grep -rn garage stormpulse/agent/*.py` turns up comments and the manifest import, nothing load-bearing.

The next test is the third integration, and I know exactly what it is: rclone, purpose-built to power an automated, turnkey migration and backup service for [Storm Buckets](https://stormdevelopments.ca/buckets). It will be the first integration written entirely against the contract, and the smallest one by far: no state loop, no detector, just two long-running jobs on a dedicated runner. If the kernel changes at all while I build it, this post was wrong twice.

I'll write that one up when it lands. The whole thing is AGPL and lives on [my own forge](https://git.stormdevelopments.ca/official-public/storm-pulse) - the contract, the guards, the two worked examples. If you're the kind of person who wants to read the most privileged program on a server before trusting it, that's exactly the audience it was written for.

Own your tools. Read your agents.
