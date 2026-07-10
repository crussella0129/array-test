# s8 Build Plan — Finalized - DO NOT EDIT

## Tasks
1. **`src/audit.rs`** — `full_audit(state_dir) -> AuditReport { problems, notes,
   counts }`: confirmations chain, every root certificate (root + cells + all_pass
   recomputed), judgments chain, evidence store re-hash. Library-first per D11.
2. **CLI** — `verify` prints the audit report; exit 0 iff zero problems.
3. **`examples/quickstart/`** — two units (dep edge; unit + closure scopes), README
   walkthrough, `judge.toml.example`; integration test runs it green (rot guard).
4. **Docs** — D19; backlog/README/sprint records.

## Out of scope
T15b durable ledger; T14; new engine features.
