# Agent Tasks — Active Backlog

Persistent across sprints (sprint-loops convention). Ordered by build dependency
(see ARCHITECTURE.md §9). Move finished items to `completed-tasks.md`.

**Toolchain (locked, D8):** Rust core engine (T1–T5, T9–T11); Python + Hypothesis for the
property tier (T7); TAP as the language-agnostic evidence contract (T6) — riteway is an
optional adapter for JS units, not a dependency of the core.

## Engine (s5+)
- [ ] **T3b — Full sandbox.** [Rust] Complete the D12/R-g gap: memory caps (rlimits),
  network isolation (namespaces/seccomp where available), filesystem read scoping.
  Upgrades a cell's determinism claim from "meta-checked" to "sandbox-guaranteed";
  the ledger should record which level applied. Also fold in R-h: a real toolchain
  pinning story replacing the "unpinned" sentinel.
- [ ] **T5b — Scope ladder.** [Rust] Generalize v1's CLOSURE-only cells (D13.1) to the
  full UNIT/DIRECT/CLOSURE/E2E ladder with per-scope resource envelopes and fail-fast
  ordering (§1.4, §5).
- [ ] **T15b — Full self-hosting workspace.** Extend the T15 milestone (one suite,
  landed s5) to a committed `selfhost/` workspace covering every test suite as units
  with real deps, producing the first durable ledger — which formally freezes the v1
  hash contexts (D9, s5 research §5).


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

## Surface (s5+)
- [ ] **T14 — sprint-loops Test-phase adapter.** Shim (sprint-loops side, or an adapter
  doc here) mapping array-test's stable outputs (`roots/R<k>.json`, ledger, green gate)
  onto sprint-loops artifacts (`test-report.md`/`failure-report.md`, phase exit).
  One-directional per D11 — array-test never learns sprint-loops exists.
