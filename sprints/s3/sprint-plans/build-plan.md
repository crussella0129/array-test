# s3 Build Plan — Finalized - DO NOT EDIT

## Tasks
1. **Hash additions** — new frozen contexts: `evidence`, `ledger-entry`,
   `ledger-genesis`, `root-leaf`, `array-root`; serde (hex string) support for `Hash`.
2. **T3 — `src/runner.rs`.** `CellSpec` → hermetic subprocess (env_clear + declared env +
   hygiene set + `ARRAY_TEST_SEED`), piped output via reader threads, wall-clock timeout
   with kill, framed evidence hashing, `run_cell` + `run_cell_checked` (determinism
   meta-check → `Verdict::Quarantined`).
3. **T4 — `src/ledger.rs`.** `DetStatus { Pass, Fail, Quarantined, TimedOut }`;
   hash-chained append-only ndjson (canonical-bytes entry hashes, genesis sentinel);
   `load`+`verify` that recomputes the chain; array root over sorted
   `{cell_key → det_status}`; `RootRecord` write/read (`roots/R<k>.json`).
4. **Docs.** D11 (embedding contract); ARCHITECTURE.md: embedding section, §6 honesty
   about v1 isolation level (R-g), det_status widening; backlog: T14 (sprint-loops
   Test-phase adapter), T3b (full sandbox: rlimit memory caps, network isolation);
   mark T3/T4 done.

## Out of scope
CLI (T11, next sprint); frontier selection (T5); mutation/fuzz tiers; the sprint-loops
side of the T14 shim.
