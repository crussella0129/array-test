# Agent Tasks — Active Backlog

Persistent across sprints (sprint-loops convention). Ordered by build dependency
(see ARCHITECTURE.md §9). Move finished items to `completed-tasks.md`.

**Toolchain (locked, D8):** Rust core engine (T1–T5, T9–T11); Python + Hypothesis for the
property tier (T7); TAP as the language-agnostic evidence contract (T6) — riteway is an
optional adapter for JS units, not a dependency of the core.

## Engine (s2+)
- [ ] **T3 — Hermetic cell runner.** [Rust] Execute one `(target, scope)` cell under
  frozen clock / pinned seed / no-I/O; emit `evidence_hash`. Include determinism
  meta-check (run twice, hashes must match) → quarantine on mismatch.
- [ ] **T4 — Confirmation ledger.** [Rust] Append-only `confirmations.ndjson`,
  hash-chained; Merkle root over `{cell_key → det_status}`; `roots/R<k>.json`.
- [ ] **T5 — Frontier selection + cache.** [Rust] Diff `code_hash`es, compute impact
  closure, derive changed `cell_key`s, reuse cached ✓ for the rest. Round cost ∝ frontier.

## Evidence (s2+)
- [ ] **T6 — TAP evidence adapter.** [Rust] Native Rust test harness emits TAP directly;
  hash into `evidence_hash`. riteway's `given/should/actual/expected` shape remains
  available as an optional adapter for JS units. (ARCHITECTURE.md §1.2, §10.6;
  decisions.md D6, D8.)

## Guarantees (s2+)
- [ ] **T7 — Property + contract tiers.** Property tier: [Python + `hypothesis`],
  invoked as a subprocess emitting TAP across the T6 boundary. Contract tier
  (pre/post/invariants): [Rust]. Record guarantee level per cell.
- [ ] **T8 — Optional formal tier.** [Rust + Kani] Model-check designated critical-unit
  invariants; attach `proof_hash`.

## Judged gate (s2+)
- [ ] **T9 — Phase J: judge gate.** [Rust orchestrator, model-agnostic judge] Independent
  judge agent reviews unit + contract + Phase-D evidence against spec, N runs vs.
  threshold; record `judgments.ndjson` entries (`judge_hash`, `critique_ref`) excluded
  from the Merkle root. (ARCHITECTURE.md §4.2, §7.3; decisions.md D7.)
- [ ] **T10 — Repair micro-loop.** [Rust] On Phase-J failure, run a Plan→Build→Test loop
  scoped to the single unit, bounded by a retry budget; escalate to sprint-level
  `failure-report.md` on exhaustion. (ARCHITECTURE.md §4.3.)

## Surface (s2+)
- [ ] **T11 — CLI + sprint-loop wiring.** [Rust] `array-test run` as a pure function of
  the tree; wire `R_k` to the sprint Test phase (Phase D then Phase J) and the
  green-root + judge-confirmed gate to Loop.
