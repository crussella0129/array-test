# s11 Test Plan — Finalized - DO NOT EDIT

No engine changes; the suite must stay at 109/109 through the fmt pass, and:

- [ ] **AC77** `cargo fmt --check` passes (CI gate is real from the first workflow run).
- [ ] **AC78** `cargo clippy --all-targets -- -D warnings` passes (the CI gate would).

## Exit condition
109/109, AC77–AC78 → s11 exits green.
