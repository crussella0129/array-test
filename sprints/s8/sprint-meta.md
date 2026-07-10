# Sprint s8 — Meta

- **Sprint:** 8
- **Title:** Verify everything; show everything — the full audit + the quickstart
- **Phase:** loop (research, build, test complete)
- **Started:** 2026-07-10
- **Exit status:** green
- **Confidence:** 1.0 (103/103; one fixture correction, no engine defects)

## Origin
Triggered by investigating a "failed background shell" that turned out to be a
`grep -c` pipeline exiting 1 on a zero count — failing because everything passed.
Doctrine recorded (D19): success must never read as failure, the dual of D14. That
audit-of-the-auditor mindset then surfaced the real gap: `verify` covered a shrinking
fraction of the state it guards.

## Definition of done
- [x] `src/audit.rs` `full_audit`: confirmations chain, every root certificate
  (recomputed root + cells + all_pass), judgments chain, evidence store re-hash;
  problems (integrity, nonzero exit) strictly separated from notes (informational).
- [x] CLI `verify` rewired onto the audit (AC65).
- [x] Tamper detection proven per surface: forged root certificate over an intact
  ledger (AC66), edited judgment line (AC67), flipped evidence byte (AC68).
- [x] `examples/quickstart/` committed (two units, dep edge, unit+closure scopes,
  walkthrough README, judge.toml.example) and guarded green by test (AC69).
- [ ] Committed & pushed.

## Notable
The tampered-root test initially tried to forge `all_pass: false → true` on R1 and
found it *already true* — correctly: the judge, not det, rejected that round, and root
certificates are Phase-D-only by design (D7). Even the test suite keeps re-teaching the
two-phase separation. The forgery that matters is the root hash; the test flips that.

## Next sprint candidates
T14 (awaiting user's side-decision), T15b (durable ledger → v1 context freeze), T7b
(contract enforcement), T8b (live Kani), T12/T13 (mutation/fuzz), T3c (FS scoping).
