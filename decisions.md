# Decisions Log

Cross-sprint architectural decision log (sprint-loops convention). Append-only.

## D1 — The regression array is a Merkle DAG of confirmations (s0)
**Context:** Schema shows a regression array growing exponentially across sprints/units.
**Decision:** Model each test as a *cell* keyed by a content hash of everything that can
affect it; record results as confirmations; commit the whole set as a Merkle root.
**Consequence:** Round cost scales with the *changed frontier*, not total history.
**Alternatives rejected:** Re-run-everything (doesn't scale); time-based caching (not
provable / not deterministic).

## D2 — Integration scope comes only from the declared DAG (s0)
**Context:** All-pairs/all-subsets integration is the combinatorial blow-up.
**Decision:** Integration cells exist only along declared `deps` edges, across a fixed
scope ladder UNIT→DIRECT→CLOSURE→E2E.
**Consequence:** Integration cells = O(edges), not O(2^units).

## D3 — Hermetic execution is mandatory (s0)
**Context:** Memoization is only valid if results are reproducible.
**Decision:** Pinned seeds, frozen clock, no ambient I/O, pinned toolchain hashed into the
cell key. A determinism meta-check quarantines non-reproducible cells.
**Consequence:** Stable keys → cache hits → the cost model in D1 holds.

## D4 — "Provable" = audit root (always) + property/contract tiers (scaled) (s0)
**Context:** User wants "provable." Full formal proof of all code is infeasible.
**Decision:** Always ship a hash-chained ledger whose green root certifies execution over
exact code. Layer property-based tests (∀-claims) and an optional model-checked formal tier
for critical units, recording the guarantee level per cell.
**Consequence:** Honest, deliverable provability with a clear upgrade path.

## D5 — One sprint = one regression round R_k (s0)
**Context:** Align the schema's R1…Rn with the sprint-loops protocol.
**Decision:** Each sprint's Test phase runs exactly one round; its green root is the gate
the next sprint reads (the schema's "loop back to current sprint").
