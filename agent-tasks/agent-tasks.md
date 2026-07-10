# Agent Tasks — Active Backlog

Persistent across sprints (sprint-loops convention). Ordered by build dependency
(see ARCHITECTURE.md §9). Move finished items to `completed-tasks.md`.

**Toolchain (locked, D8):** Rust core engine (T1–T5, T9–T11); Python + Hypothesis for the
property tier (T7); TAP as the language-agnostic evidence contract (T6) — riteway is an
optional adapter for JS units, not a dependency of the core.

## Engine (s4+)
- [ ] **T3b — Full sandbox.** [Rust] Complete the D12/R-g gap: memory caps (rlimits),
  network isolation (namespaces/seccomp where available), filesystem read scoping.
  Upgrades a cell's determinism claim from "meta-checked" to "sandbox-guaranteed";
  the ledger should record which level applied.
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
- [ ] **T12 — Frontier-scoped mutation testing.** [Rust + cargo-mutants] Mutation score
  as guarantee-level metadata per confirmation; only dirty units re-mutate — scores are
  memoized by `code_hash`, the same economics as the regression frontier. (D10;
  s2 research report §2.5.)
- [ ] **T13 — Fuzz tier (opt-in).** [Rust + cargo-fuzz] Coverage-guided fuzzing per
  unit; corpus is a content-addressed fixture set, so corpus growth re-keys cells
  naturally. (D10; s2 research report §2.6.)

## Judged gate (s2+)
- [ ] **T9 — Phase J: judge gate.** [Rust orchestrator, model-agnostic judge] Independent
  judge agent reviews unit + contract + Phase-D evidence against spec, N runs vs.
  threshold; record `judgments.ndjson` entries (`judge_hash`, `critique_ref`) excluded
  from the Merkle root. (ARCHITECTURE.md §4.2, §7.3; decisions.md D7.)
- [ ] **T10 — Repair micro-loop.** [Rust] On Phase-J failure, run a Plan→Build→Test loop
  scoped to the single unit, bounded by a retry budget; escalate to sprint-level
  `failure-report.md` on exhaustion. (ARCHITECTURE.md §4.3.)

## Surface (s4+)
- [ ] **T11 — CLI.** [Rust] `array-test run` as a pure function of the tree; the
  standalone binary an embedder without Rust linkage uses. Consumer-agnostic per D11.
- [ ] **T14 — sprint-loops Test-phase adapter.** Shim (sprint-loops side, or an adapter
  doc here) mapping array-test's stable outputs (`roots/R<k>.json`, ledger, green gate)
  onto sprint-loops artifacts (`test-report.md`/`failure-report.md`, phase exit).
  One-directional per D11 — array-test never learns sprint-loops exists.
