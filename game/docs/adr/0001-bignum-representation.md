# ADR 0001 — Game numbers are a custom big-float {mantissa, exponent}

**Status:** Accepted
**Date:** 2026-06-30

## Context

Every resource, rate, and cost in the game is "a number." An idle game with a
long-term prestige loop drives those numbers far past anything a native integer
holds — published incremental games routinely reach 1e1000 and beyond. The sim
core is Rust→WASM, so the choice is ours to make from first principles. The
representation is the single most foundational data structure: every formula and
every saved value depends on it, and changing it later means rewriting all of
them.

## Decision

Represent every game number as a custom big-float: an `f64` mantissa plus an
`i64` exponent (scientific-notation, in the lineage of `break_infinity.js`). The
ceiling is effectively infinite. The sim core owns the type and all its
arithmetic, comparison, and formatting helpers.

## Alternatives considered

- **`u128` integers** — native, exact, trivial, but the ~3.4e38 ceiling is a
  hard design wall a prestige game will hit. Rejected: the cap can't be lifted
  without rewriting every formula.
- **Arbitrary-precision (`num-bigint`)** — exact and truly unbounded, but
  heap-allocates per operation and slows as values grow, buying exactness the
  player never sees. Rejected: cost without benefit at idle-game scale.

## Consequences

- Numbers lose exactness past 2^53 of mantissa precision. Acceptable: the game
  never *displays* that precision, and idle-game balance never depends on it.
- All arithmetic goes through the big-float helpers, never raw `f64`/`u128`.
  This is a discipline the codebase must hold from line one.
- The save format stores numbers in this representation (see the forthcoming
  save-schema ADR), so the choice also shapes serialization.
