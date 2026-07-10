# s5 Test Plan — Finalized - DO NOT EDIT

AC1–AC38 stay green. New checks:

## T6 TAP adapter
- [ ] **AC39** libtest-shaped output (with `running N tests` / timing noise) → minimal
  sorted TAP 13; noise lines absent from the output.
- [ ] **AC40** A `FAILED` line → `not ok` point and nonzero wrapper exit; all-ok →
  exit 0.
- [ ] **AC41** `ignored` → `ok … # SKIP`; a nonzero inner exit with no parsed failure →
  synthetic `not ok` appended (silence never reads as success).
- [ ] **AC42** Two runs whose inner output differs only in timing noise produce
  byte-identical wrapper stdout — the determinism the meta-check needs, produced at the
  source.

## T15 self-hosting
- [ ] **AC43** A round whose cell wraps our own `t2_dag_resolver` suite via the tap
  adapter is **Executed + Pass** (not quarantined) and the round certifies green.
- [ ] **AC44** `array-test verify` exits 0 on the self-host state; a second round
  **reuses** the confirmation with an identical root.

## Exit condition
AC1–AC44 green, clippy clean → s5 exits green.
