# s12 Test Plan — Finalized - DO NOT EDIT

AC1–AC78 stay green. New checks (unix, scripted mutators):

- [ ] **AC79** A unit whose test actually checks content kills every mutant: score
  100, `strong = true`.
- [ ] **AC80** A unit whose test asserts nothing lets every mutant survive: score 0,
  `strong = false`, and the mutation run reports not-all-strong (the pathology the
  tier exists to expose).
- [ ] **AC81** Memoization: an unchanged workspace re-mutated ⇒ zero mutator
  invocations (scores served from cache keyed by code/mutator/baseline-root).
- [ ] **AC82** Changing the unit's src (or its detection surface) re-mutates —
  invocations observed again.
- [ ] **AC83** `mutations.ndjson` chain-verifies; `full_audit` counts it; a tampered
  line fails the audit.
- [ ] **AC84** CLI `array-test mutate` exits 0 on all-strong and 1 otherwise.

## Exit condition
AC1–AC84 green, fmt clean, clippy clean → s12 exits green; PR to main.
