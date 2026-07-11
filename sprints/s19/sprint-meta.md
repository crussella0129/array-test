# Sprint s19 — Meta
- Sprint: 19
- Title: Function decomposition (F3)
- Phase: loop
- Exit status: green
- Confidence: 1.0 (130 pass / 3 ignored; clippy -D warnings + fmt clean; behavior-preserving)

## Done
- full_audit (153 -> ~18): audit_confirmations / audit_roots / audit_sidecar_chains /
  audit_evidence, one shared AuditReport, phase-1 returns the entries later phases reuse.
- run_round (131 -> 81): resolve_cell lifts the per-cell skip/cache/run/store decision out
  of the tier-gating loop.
- run_mutation (139 -> ~55): MutationRun<'a> context struct + score_unit + evaluate_mutant
  (MutantOutcome enum). Struct chosen to avoid clippy::too_many_arguments and re-threading
  seven run-wide invariants.

## Deferred
- run_cell (128, runner.rs): fork/exec + namespace sandbox; privileged paths are
  #[ignore]-gated, higher regression risk per line — left for a focused pass (D31).

## Verification
Behavior-preserving: no byte/ledger/key change. t16_audit, t12_mutation, round tests are
the witnesses. See D31.

## Next
s21 (F36/F38–F42 perf), s22 (F28–F35 docs/archival), or a focused run_cell pass.
