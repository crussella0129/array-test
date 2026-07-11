# Sprint s20 — Meta
- Sprint: 20
- Title: Security hardening — id validation (F18), defensive containment (F16), trust-boundary docs (F17/F21)
- Phase: loop
- Exit status: green
- Confidence: 1.0 (129 pass / 3 ignored; clippy -D warnings + fmt clean; zero frozen surfaces)

## Done
- F18 (real): validate_unit_id at manifest load — reject separators, `..`, leading dot,
  control/whitespace, empty. Dotted namespacing stays legal. Author-controlled `id` reaches
  mutation work-dir paths, so this is the substantive fix of the security cluster.
- F16 (defensive, premise corrected): safe_state_path guards the repair-loop join against
  absolute / non-Normal components. critique_ref is engine-generated and cannot traverse
  today — this is a guard for a future disk-loaded judgment, not a live-vuln fix.
- F17: run_repair trust-boundary doc — operator-authored command at test trust level, no
  attacker-controlled argv/env, unsandboxed by design (next det round re-runs sandboxed).
- F21: make_root_readonly Safety doc — caller must already be in a fresh private mount ns.

## Corrections to the plan
- F16's "attacker-supplied critique_ref joined without validation" was false: the ref is
  engine-generated (`ledger/critiques/<64-hex>/N.md`). Kept as defense-in-depth; framing
  corrected in D29.

## Coverage added
- Manifest id-traversal rejection test (a/b, .., ../escape, .hidden, a\b, a<TAB>b; dotted OK).
- safe_state_path accept/reject unit test.

## Next
s18 (F2 enum-ify manifest scopes), s19 (F3 function decomposition), s21 (F36/F38–F42 perf),
s22 (F28–F35 docs/archival). Deferred: T8b live Kani (env-gated), T14 sprint-loops adapter.
