# s14 Test Plan — Finalized - DO NOT EDIT

- [ ] AC90 RO cell cannot write anywhere (incl. /tmp), can read, host unaffected
  (capability-gated).
- [ ] AC91 the flag is part of the cell identity (declared env is hashed).
- [ ] AC92 quickstart contract-audit enforces dependency contracts, green, 3 cells.

Exit: AC1–AC92 green, fmt+clippy clean.
