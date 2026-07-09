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

## Evidence (s1+)
- [ ] **T6 — riteway/TAP evidence adapter.** Author `tests/` in riteway's
  `given/should/actual/expected` shape; run via riteway; capture TAP output; hash into
  `evidence_hash`. (See ARCHITECTURE.md §1.2, §10.6; decisions.md D6.)

## Guarantees (s2+)
- [ ] **T7 — Property + contract tiers.** Generator-based property runner with shrinking;
  enforce contract pre/post/invariants per cell; record guarantee level.
- [ ] **T8 — Optional formal tier.** Encode designated critical-unit invariants for a
  model checker / SMT; attach `proof_hash`.

## Judged gate (s2+)
- [ ] **T9 — Phase J: judge gate.** Independent judge agent reviews unit + contract +
  Phase-D evidence against spec, N runs vs. threshold; record `judgments.ndjson` entries
  (`judge_hash`, `critique_ref`) excluded from the Merkle root. (ARCHITECTURE.md §4.2,
  §7.3; decisions.md D7.)
- [ ] **T10 — Repair micro-loop.** On Phase-J failure, run a Plan→Build→Test loop scoped
  to the single unit, bounded by a retry budget; escalate to sprint-level
  `failure-report.md` on exhaustion. (ARCHITECTURE.md §4.3.)

## Surface (s2+)
- [ ] **T11 — CLI + sprint-loop wiring.** `array-test run` as a pure function of the tree;
  wire `R_k` to the sprint Test phase (Phase D then Phase J) and the green-root +
  judge-confirmed gate to Loop.
