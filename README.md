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

## Sprint-loop state
- `decisions.md` — architectural decision log (D1–D5).
- `confidence.txt` — sprint-loop confidence throttle.
- `agent-tasks/` — active backlog + completion log.
- `sprints/s0/` — current sprint (design): research report + locked build/test plans.

## Status
**Sprint s0 (design)** complete: schema analyzed, architecture specified, state machine
scaffolded. **Sprint s1** builds the substrate — content addressing + schemas (T1) and the
integration DAG resolver (T2). See `agent-tasks/agent-tasks.md`.
