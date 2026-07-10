# s8 Research Report — Verify Everything; Show Everything

## 1. Input
Two findings drive this sprint:

1. **A shell-hygiene incident with a doctrine-shaped moral.** A background check shell
   "failed" after s7 — investigation showed it was `cargo test | grep -c FAILED`
   printing `0` (the healthy outcome) while `grep -c` exits 1 on zero matches: the
   failure signal fired precisely because everything was perfect. This is the dual of
   D14's "silence never reads as success": **success must never read as failure.** An
   evidence protocol must be honest in both directions. (Fix applied to our own dev
   loop; principle recorded here because array-test's own CLI exit codes are consumer
   contract surface — audited below.)
2. **The verifier is behind the ledger.** `verify` checks the confirmations chain and
   the *latest* root only. Since then the state grew judgments (hash-chained), critique
   transcripts, an evidence store, and per-round certificates — none audited. D11
   promises "anyone holding the state can re-verify with zero trust in the runner";
   that promise currently covers a shrinking fraction of the state.

## 2. Design (→ D19)

### 2.1 Full audit as a library function
`audit::full_audit(state_dir) -> AuditReport` — library-first (D11: embedders get the
trust tool, the CLI is just one caller). Checks:

- **Confirmations chain** — every entry re-hashed, every link (existing).
- **Every root certificate** — for each `roots/R<k>.json`: recompute the root from that
  round's ledger entries; recompute `cells` and `all_pass`; all three must match the
  certificate. (Catches a tampered/forged certificate even when the ledger is intact.)
- **Judgments chain** — full re-hash + linkage (the mechanism exists in `judge.rs`;
  verify never called it).
- **Evidence store** — every stored file's bytes must re-hash to its filename. (The
  store is content-addressed; a mis-hashing file is tampering, full stop.) Entries
  whose evidence is absent from the store are counted informationally, not failed —
  quarantined/skipped evidence is legitimately never stored.

The report separates **problems** (integrity violations → nonzero exit) from **notes**
(informational counts). Empty state dirs audit clean-but-noted.

### 2.2 A committed quickstart example, guarded by CI
`examples/quickstart/` — a real two-unit workspace (dependency edge, unit+closure
scopes, machine-independent `/bin/sh` cells) with a walkthrough README and a commented
`judge.toml.example`. An integration test runs a round over it and asserts green: the
example cannot rot without failing the suite. This is the D11 adoption surface — the
first thing a standalone user runs.

Note: the example's *state* stays out of the repo (tempdir in tests). The first durable
committed ledger — and the formal v1 context freeze — remains T15b's milestone.

## 3. Recommendation
Build `src/audit.rs`; rewire `cmd_verify` onto it; tamper tests per audited surface
(root certificate, judgment line, evidence bytes); commit the quickstart + its guard
test. Record D19.
