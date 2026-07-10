# array-test

A system for agentic software test composition, implementation, and recording.

`array-test` models the regression suite as a **Merkle DAG of confirmations**: every test
is a content-addressed *cell* that only re-runs when something that affects it changes, and
a single green root hash certifies that all confirmations hold for exactly the current
code. This tames the exponential growth of regression complexity while staying
deterministic, code-based, and provable.

It is derived from a hand-drawn agentic testing schema (units built per sprint; an
integration/regression array that travels down, out, and backwards with a confirmation at
each step) and built using the [sprint-loops](https://github.com/crussella0129/sprint-loops)
protocol (Research → Plan → Build → Test → Loop; the filesystem is the state machine).

## Documentation
- [`docs/SCHEMA-ANALYSIS.md`](docs/SCHEMA-ANALYSIS.md) — reading of the original schema.
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — the deterministic/provable design.

## Toolchain
**Rust** core engine (content addressing, DAG resolver, hermetic runner, ledger/Merkle
root, judge gate, CLI) + **Python (Hypothesis)** for property-based tests, connected by
**TAP** as a language-agnostic evidence contract (`decisions.md` D8).

## Sprint-loop state
- `decisions.md` — architectural decision log (D1–D8).
- `confidence.txt` — sprint-loop confidence throttle.
- `agent-tasks/` — active backlog + completion log.
- `sprints/s0/` — design sprint (closed, green): research report + locked build/test plans.
- `sprints/s1/` — closed, green: research (riteway investigation, two-phase confirmation
  gate, toolchain lock) + T1/T2 built and tested.
- `sprints/s2/` — closed, green: testing-practice survey (10 topics, adoption map in D10)
  + refactor: domain-separated hashing (D9), filesystem determinism, manifest validation,
  `topo_order()`.

## Building
```
cargo build
cargo test      # AC1-AC8 (sprints/s1/sprint-plans/test-plan.md)
cargo clippy --all-targets
```

## Notable design points
- Regression is a **Merkle DAG of confirmations**: content-addressed cells, frontier-only
  re-runs, a hash-chained ledger with a verifiable green root (`docs/ARCHITECTURE.md`).
- Confirmation is a **two-phase gate**: a deterministic, reproducible test phase (Phase D,
  what the Merkle root certifies) *and* an independent judge-agent review (Phase J, audited
  but not rooted). A judge rejection triggers a repair micro-loop scoped to the single unit,
  not the whole sprint (`docs/ARCHITECTURE.md` §4).
- Test authoring and evidence format follow
  [riteway](https://github.com/crussella0129/riteway)'s `given/should/actual/expected` +
  TAP conventions.

## Status
Sprints **s0 (design)**, **s1 (substrate)**, and **s2 (survey + hardening refactor)** all
closed green. Implemented and tested (`src/hash.rs`, `src/manifest.rs`, `src/contract.rs`,
`src/dag.rs`): domain-separated `code_hash`/`cell_key` content addressing (frozen
`array-test/v1/...` contexts, RFC 6962-style leaf/node prefixes), cross-platform
deterministic file hashing, validated manifest/contract schemas, and the integration DAG
resolver (forward closure, reverse impact closure, deterministic `topo_order()`). Next up:
T3 (hermetic cell runner) + T4 (confirmation ledger / Merkle root) — the first sprint that
can actually run an `R_k`, at which point the v1 hash contexts freeze for good. See
`agent-tasks/agent-tasks.md`.
