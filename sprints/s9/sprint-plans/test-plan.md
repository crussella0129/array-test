# s9 Test Plan — Finalized - DO NOT EDIT

AC1–AC69 stay green through the re-key (they assert properties, not hash values).
New checks:

- [ ] **AC70** Quarantined cells persist BOTH runs' evidence in the store,
  content-addressed (F9); the audit stays clean over a state containing them.
- [ ] **AC71** Round numbering survives a lost certificate: run, delete `R1.json`,
  run again → the new round is 2 (ledger-derived), never a reused 1; the audit notes
  the certificate-less round without failing (F10, F16).
- [ ] **AC72** A manifest without `sprint` loads (F13).
- [ ] **AC73** The skipped-evidence sentinel differs from any `EVIDENCE`-domain hash of
  the same bytes (F8, domain separation regression guard).

## Exit condition
AC1–AC73 green, clippy clean (zero allows added) → s9 exits green.
