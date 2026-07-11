# s11 Build Plan — Finalized - DO NOT EDIT

1. `cargo fmt` pass (mechanical) so CI can gate on `fmt --check`.
2. `.github/workflows/ci.yml`: fmt check, build, test (includes the rot guard over the
   committed founding ledger), clippy `-D warnings`.
3. `docs/TEMPLATE.md`: the two layers, instantiation steps, genesis ritual,
   never-relayout rule.
4. README template section; D22; sprint records.

## Out of scope
Flipping the GitHub template setting (human step); deferred tiers; T14.
