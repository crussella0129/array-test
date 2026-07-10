# s10 Build Plan — Finalized - DO NOT EDIT

## Tasks
1. `selfhost/units/{tap-cli,run-cli,verify-cli}` — scripts in `src/check.sh`, relative
   PATH to `target/debug`, unit/unit/closure scopes, deps edge run→verify.
2. `selfhost/units/toolchain.lock` from this environment's `rustc -vV`.
3. Run two rounds via the CLI; commit `selfhost/state/` (ledger, certificates R1/R2,
   cache, evidence).
4. `tests/t15b_durable.rs` — rot guard: `full_audit` clean over the committed state;
   R1/R2 certificates present with identical roots; machine-independent (pure file
   verification, no execution).
5. Freeze declaration: D21; `hash.rs` domain docs marked FROZEN; README status;
   `selfhost/README.md`.
6. Version 1.0.0.

## Out of scope
Deferred tiers (extend post-freeze per D20); T14.
