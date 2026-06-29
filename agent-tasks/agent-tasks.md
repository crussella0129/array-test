# Agent Tasks — Active Backlog

Persistent across sprints (sprint-loops convention). Ordered by build dependency
(see ARCHITECTURE.md §9). Move finished items to `completed-tasks.md`.

## Foundation (Sprint s0 candidates)
- [ ] **T1 — Content addressing & schemas.** Define `manifest.toml` + `contract.toml`
  schemas; implement `code_hash = H(src ‖ contract)` and `cell_key` (§2). Deterministic,
  stable, documented.
- [ ] **T2 — Integration DAG resolver.** Parse `deps`, build `dag.json`, detect cycles,
  expose forward (closure / "down") and reverse (impact / "backwards") traversals.

## Engine (s1+)
- [ ] **T3 — Hermetic cell runner.** Execute one `(target, scope)` cell under frozen
  clock / pinned seed / no-I/O; emit `evidence_hash`. Include determinism meta-check
  (run twice, hashes must match) → quarantine on mismatch.
- [ ] **T4 — Confirmation ledger.** Append-only `confirmations.ndjson`, hash-chained;
  Merkle root over `{cell_key → status}`; `roots/R<k>.json`.
- [ ] **T5 — Frontier selection + cache.** Diff `code_hash`es, compute impact closure,
  derive changed `cell_key`s, reuse cached ✓ for the rest. Round cost ∝ frontier.

## Guarantees (s2+)
- [ ] **T6 — Property + contract tiers.** Generator-based property runner with shrinking;
  enforce contract pre/post/invariants per cell; record guarantee level.
- [ ] **T7 — Optional formal tier.** Encode designated critical-unit invariants for a
  model checker / SMT; attach `proof_hash`.

## Surface (s2+)
- [ ] **T8 — CLI + sprint-loop wiring.** `array-test run` as a pure function of the tree;
  wire `R_k` to the sprint Test phase and the green-root gate to Loop.
