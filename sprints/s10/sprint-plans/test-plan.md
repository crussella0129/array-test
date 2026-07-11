# s10 Test Plan — Finalized - DO NOT EDIT

AC1–AC73 stay green. New checks:

- [ ] **AC74** The selfhost round runs green end-to-end via the CLI (executed live in
  this sprint; its artifacts are the committed state).
- [ ] **AC75** Rot guard: `full_audit` over the **committed** `selfhost/state` is clean
  on every test run; R1 and R2 certificates exist, R2 is all-reused history
  (`reused = true` entries), and both roots are identical.
- [ ] **AC76** The committed ledger's entries all carry v1-context hashes that verify —
  i.e. the freeze condition is real, not declared.

## Exit condition
AC1–AC76 green, clippy clean → s10 exits green; v1 FROZEN; version 1.0.0.
