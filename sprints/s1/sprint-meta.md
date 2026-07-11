# Sprint s1 — Meta

- **Sprint:** 1
- **Title:** Substrate: content addressing, DAG resolver, riteway/TAP evidence, two-phase gate design
- **Phase:** loop (build + test complete)
- **Started:** 2026-07-09
- **Exit status:** green
- **Confidence:** 1.0 (all AC1-AC8 passed first try; no patches required)

## Goal
Build the substrate every later regression cell depends on (content addressing, DAG,
TAP evidence), informed by the riteway research and the two-phase confirmation gate design
locked in this sprint's research.

## Definition of done
- [x] Research: riteway investigated, two-phase gate (Phase D + Phase J) designed and
  written into `docs/ARCHITECTURE.md` §4/§7/§8/§10.
- [x] `decisions.md` D6, D7 recorded.
- [x] `agent-tasks/agent-tasks.md` reordered with T6 (evidence adapter), T9/T10 (judge
  gate + repair micro-loop).
- [x] Toolchain assessed (Rust vs. Python vs. Node/TS) and locked: Rust core engine +
  Python/Hypothesis property tier. `decisions.md` D8 recorded; resolves R-d.
- [x] T1 content addressing + schemas implemented (`src/hash.rs`, `src/manifest.rs`,
  `src/contract.rs`). AC1-AC4 green (15 tests, incl. extra edge cases).
- [x] T2 integration DAG resolver implemented (`src/dag.rs`, petgraph-backed). AC5-AC8
  green (10 tests).
- [x] `cargo build`/`cargo clippy --all-targets` clean, no warnings.
- [ ] Committed & pushed.

## R1 — first real regression round
25/25 tests pass on the first run; no failures, no repairs needed. Per D5, this closes
s1's Test phase green. `array/ledger/roots/R1.json` is not yet written — the ledger/root
mechanism itself is T4 (s2), so R1 here means "s1's own test-plan acceptance checks," not
yet a run of the array-test engine against itself. That bootstrapping (using array-test's
own ledger to certify its own tests) becomes possible once T3/T4 exist.

## Next sprint (s2) preview
T3 hermetic cell runner + determinism meta-check; T4 confirmation ledger + Merkle root —
the first sprint that can actually run an `R_k`.
