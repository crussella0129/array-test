# s7 Test Plan — Finalized - DO NOT EDIT

AC1–AC55 stay green. New checks (unix):

## Guarantee levels + evidence store
- [ ] **AC56** `guarantee` is validated (unknown value rejected), defaults to
  `example`, is recorded in ledger entries, and changing it re-keys the cell.
- [ ] **AC57** Every executed cell's evidence bytes land in the store under their hash,
  and the stored bytes re-hash to `evidence_hash`.

## T7 property tier
- [ ] **AC58** A real Hypothesis property cell (derandomized) passes hermetically with
  `guarantee = "property"` recorded — deterministic across the meta-check's two runs
  (skip with a note if python3/hypothesis is absent on the host).

## T9 judge gate
- [ ] **AC59** With `judge.toml` present, a det-green round whose unit the judge rates
  below `min_rating` is judged NOT green; critique transcript exists; the judgments
  ledger chain-verifies.
- [ ] **AC60** Judged verdicts are cached: an unchanged cell + unchanged judge is not
  re-invoked on the next run (observed via an invocation-marker file).
- [ ] **AC61** Changing the judge config (new `judge_hash`) re-judges the same cell.
- [ ] **AC62** Phase J only runs over a det-green round: a det-red round returns
  without invoking the judge.

## T10 repair micro-loop
- [ ] **AC63** A rejected unit is repaired by the repair command, the next attempt
  re-runs exactly the re-keyed cells (a new det round), the judge passes, and the
  overall outcome is green — with each attempt visible as a numbered round in history.
- [ ] **AC64** With no repair config (or budget exhausted), a judged-red outcome writes
  `ledger/failures/R<k>-judgment.md` referencing the critiques, and the CLI exits
  nonzero.

## Exit condition
AC1–AC64 green, clippy clean → s7 exits green.
