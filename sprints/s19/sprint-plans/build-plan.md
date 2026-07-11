# s19 Build Plan
1. audit.rs: extract audit_confirmations (returns entries), audit_roots, audit_sidecar_chains,
   audit_evidence; full_audit becomes the 4-call narrative.
2. round.rs: extract resolve_cell(paths, plan, gate_broken, skipped_evidence) -> (status,
   evidence, kind); run_round calls it in the tier loop.
3. mutation.rs: MutationRun<'a> context + score_unit + evaluate_mutant (MutantOutcome enum);
   run_mutation builds the context and loops.
4. Verify: build, full suite (esp. t16_audit, t12_mutation), clippy -D warnings, fmt --check.
5. Defer run_cell with a documented rationale (D31).
