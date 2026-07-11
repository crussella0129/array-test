# Sprint s3 — Meta

- **Sprint:** 3
- **Title:** Hermetic runner + confirmation ledger, under the embedding contract
- **Phase:** loop (research, build, test complete)
- **Started:** 2026-07-10
- **Exit status:** green
- **Confidence:** 1.0 — with a note: one real bug was caught and fixed *in-sprint by the
  sprint's own test plan* (see Notable). Tests pass 57/57; no failure report needed.

## Goal
Make the array runnable: T3 (hermetic cell runner) + T4 (hash-chained ledger + array
root), designed under the new constraint that array-test is standalone-first and
sprint-loops is merely its first consumer (D11).

## Definition of done
- [x] Research: embedding contract defined (D11); runner/ledger design decisions +
  honesty about v1 isolation level (D12, gap R-g).
- [x] T3 `src/runner.rs`: cleared env + hygiene set + seed, framed evidence hashing,
  wall-clock envelope with process-group kill, determinism meta-check → quarantine.
- [x] T4 `src/ledger.rs`: canonical-bytes chained entries, genesis sentinel, full-chain
  verification, order/timestamp-independent array root, `roots/R<k>.json` certificates.
- [x] `DetStatus` = Pass | Fail | Quarantined | TimedOut — D10's visibility requirement
  in the schema; only all-Pass is green; empty cell set is not green.
- [x] New frozen contexts: evidence, ledger-entry, ledger-genesis, root-leaf, array-root.
- [x] AC19–AC28 green (21 new tests; 57 total), clippy clean.
- [ ] Committed & pushed.

## Notable
AC22 (envelope breach) caught a real bug on first run: killing the direct child (`sh`)
left its grandchild (`sleep 30`) alive holding the output pipes — the t3 suite took
30.01s instead of ~0.1s. Fix: run each cell as its own **process group** and kill the
group. The test was also strengthened to assert the *call's* wall time, since the
original assertion measured a duration captured before the hang and passed anyway.
Lesson recorded: assertions must measure the observable a failure would actually
corrupt.

## Next sprint (s4) preview
T5 frontier selection + cache (wires hash/dag/runner/ledger into an actual `R_k`) and
T11 CLI — after which the first self-hosted round can run: array-test running its own
test suite as cells and certifying its own root.
