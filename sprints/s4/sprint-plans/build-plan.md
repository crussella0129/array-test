# s4 Build Plan — Finalized - DO NOT EDIT

## Tasks
1. **Schema** — `manifest.toml` gains optional `[test]` (`command`, `env`,
   `timeout_secs`); validation: declared command must be non-empty.
2. **Hash** — contexts `array-test/v1/test-def`, `array-test/v1/fixtures`; `Hash::hex()`
   for filenames.
3. **Ledger** — `append_entry(..., reused: bool)` with the flag inside the canonical
   bytes; `append` delegates with `reused = false`.
4. **T5 — `src/round.rs`.** `load_workspace` (units dir → manifests + code_hashes + DAG,
   duplicate-id rejection); `plan_round` (topo order, CLOSURE-scope cell keys,
   test_def/fixtures hashing); cache read/write (Pass/Fail only); `run_round`
   (cache-aware execution via `run_cell_checked`, per-round entries, round root,
   `roots/R<k>.json`, `RoundReport` with executed/reused/status detail).
5. **T11 — `src/main.rs`.** `array-test run --units <dir> --state <dir> [--round N]
   [--seed N] [--toolchain-hash H]` (exit 0 iff green) and `array-test verify --state
   <dir>` (chain + latest-root integrity). Hand-rolled args.
6. **Docs.** D13 (round semantics: 2.1–2.4), R-h gap, T15 backlog; mark T5/T11 done.

## Out of scope
Scope ladder beyond CLOSURE; fixtures store; toolchain auto-pinning (R-h); T15
self-hosting; Phase J.
