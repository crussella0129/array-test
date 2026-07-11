# s8 Test Plan — Finalized - DO NOT EDIT

AC1–AC64 stay green. New checks:

- [ ] **AC65** `full_audit` passes on a healthy judged state (confirmations, all roots,
  judgments, evidence store) with zero problems; CLI `verify` exits 0.
- [ ] **AC66** A tampered root certificate (edited `R<k>.json`) is detected even though
  the ledger itself is intact; CLI exits nonzero.
- [ ] **AC67** A tampered judgments line is detected.
- [ ] **AC68** A tampered evidence file (bytes no longer match the content address) is
  detected.
- [ ] **AC69** The committed `examples/quickstart` workspace runs a green round (the
  example cannot rot silently).

## Exit condition
AC1–AC69 green, clippy clean → s8 exits green.
