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
- `decisions.md` — architectural decision log (D1–D7).
- `confidence.txt` — sprint-loop confidence throttle.
- `agent-tasks/` — active backlog + completion log.
- `sprints/s0/` — design sprint (closed, green): research report + locked build/test plans.
- `sprints/s1/` — current sprint: research (riteway investigation, two-phase confirmation
  gate) + locked build/test plans for the content-addressing and DAG substrate.

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
**Sprint s0 (design)** closed green. **Sprint s1** in progress: research + plan locked for
T1 (content addressing + schemas) and T2 (integration DAG resolver) — the substrate every
later cell key depends on. See `agent-tasks/agent-tasks.md`.
