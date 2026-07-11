# s13 Test Plan — Finalized - DO NOT EDIT

AC1–AC84 stay green (fixtures absent ⇒ sentinel ⇒ existing keys unchanged). New:

- [ ] **AC85** Adding/changing files under `<unit>/fixtures/` re-keys the unit's cells
  (executed, not reused); absent dir keeps the sentinel (prior roots reproduce).
- [ ] **AC86** A fuzzer finding (exit 65 + corpus write) is recorded in the sidecar,
  moves `fixtures_hash`, and the next round re-executes the unit's cells against the
  grown corpus.
- [ ] **AC87** A clean fuzz result is cached: unchanged unit+corpus ⇒ no fuzzer
  invocation on the next `fuzz` run; changing src re-fuzzes.
- [ ] **AC88** `fuzz.ndjson` chain-verifies; audit counts it; tampering detected.
- [ ] **AC89** CLI `array-test fuzz` exits 0 when clean, 1 when findings were produced.

## Exit condition
AC1–AC89 green, fmt+clippy clean → s13 green; PR to main.
