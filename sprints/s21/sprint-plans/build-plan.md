# s21 Build Plan
1. audit.rs: merge the two evidence-dir enumerations in audit_evidence into one pass,
   building the stored stem set inside the hash-check loop. Preserve exact semantics.
2. README.md: update Sprint-loop state (D1–D31, sprints s0–s20 grouped) and Status
   (130 tests, s0–s20, fuzz tier). Verify no stale markers remain across docs/.
3. Verify: full suite (esp. t16_audit), clippy -D warnings, fmt --check.
