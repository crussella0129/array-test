# Sprint s1 — Meta

- **Sprint:** 1
- **Title:** Substrate: content addressing, DAG resolver, riteway/TAP evidence, two-phase gate design
- **Phase:** plan (research complete, build-plan locked below; build not yet started)
- **Started:** 2026-07-09
- **Exit status:** open
- **Confidence:** 1.0 (carried from s0; no failures yet)

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
- [ ] T1 content addressing + schemas implemented.
- [ ] T2 integration DAG resolver implemented.
- [ ] Committed & pushed.

## Next sprint (s2) preview
T3 hermetic cell runner + determinism meta-check; T4 confirmation ledger + Merkle root —
the first sprint that can actually run an `R_k`.
