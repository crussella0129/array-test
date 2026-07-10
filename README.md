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
- `sprints/s3/` — closed, green: embedding contract (D11), hermetic cell runner (T3,
  v1 isolation level per D12), hash-chained confirmation ledger + array root (T4).

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

## Embedding
array-test is **library-first and consumer-agnostic** (D11). It is being built to power
the Test phase of the [sprint-loops](https://github.com/crussella0129/sprint-loops)
protocol, but the core never references sprint-loops — or any consumer. Integrate against
the stable outputs: the all-PASS green gate, `roots/R<k>.json` round certificates, the
independently re-verifiable hash-chained `confirmations.ndjson`, and hash-committed TAP
evidence. Anyone holding the ledger file can re-verify the chain and root with zero trust
in the runner.

## Status
Sprints **s0 (design)**, **s1 (substrate)**, **s2 (survey + hardening refactor)**, and
**s3 (runner + ledger)** all closed green — 57 tests. Implemented: domain-separated
`code_hash`/`cell_key` content addressing (frozen `array-test/v1/...` contexts, RFC
6962-style leaf/node prefixes), validated manifest/contract schemas, the integration DAG
resolver (forward/impact closures, deterministic `topo_order()`), the hermetic cell
runner (cleared env + seed, evidence hashing, wall-clock envelope with process-group
kill, run-twice determinism meta-check → visible quarantine), and the hash-chained
confirmation ledger with reproducible array roots. Next up: T5 (frontier selection +
cache) and T11 (CLI) — the first self-hosted `R_k`. See `agent-tasks/agent-tasks.md`.
