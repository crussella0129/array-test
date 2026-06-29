# s0 Test Plan — Finalized - DO NOT EDIT

s0 produces no executable code, so there is no regression round to run (R0 = empty array).
"Tests" here are **acceptance checks** on the design artifacts.

## Acceptance checks
- [x] **AC1** `SCHEMA-ANALYSIS.md` accounts for every labelled element of the drawing
  (U, sprint columns, Integration axis, grid, R1–R6, ✓, loop-back, real-time/history).
- [x] **AC2** `ARCHITECTURE.md` gives a concrete mechanism for each of: down, out,
  backwards, confirmation-at-each-step, loop-back, real-time/history.
- [x] **AC3** Each exponential-growth source has a named lever with a complexity claim.
- [x] **AC4** "Provable" is defined operationally (audit root + property/formal tiers),
  not aspirationally.
- [x] **AC5** Scaffolding matches the sprint-loops layout (`decisions.md`, `confidence.txt`,
  `agent-tasks/`, `sprints/s0/...`).

## When the engine exists (forward-looking, for s1's test-plan)
The first real regression round R1 must demonstrate: (a) a cell re-runs iff its `cell_key`
changes; (b) an unchanged cell reuses its cached ✓ with zero execution; (c) the Merkle
root recomputes deterministically; (d) the determinism meta-check catches a deliberately
non-hermetic cell.

## Result
All acceptance checks pass → s0 exits green. Confidence held at 1.0 (no failures, no
patches required).
