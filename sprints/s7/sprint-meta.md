# Sprint s7 — Meta

- **Sprint:** 7
- **Title:** The guarantee tiers — property, proved, judged, repaired
- **Phase:** loop (research, build, test complete)
- **Started:** 2026-07-10
- **Exit status:** green
- **Confidence:** 1.0 (98/98 tests; every new AC green on the first full run; one
  clippy finding resolved by refactor rather than suppression)

## Goal
User-directed (direction 3): make "provable" and "agentic" code, not design sections —
T7 property tier, T8 proved schema, T9 Phase-J judge gate, T10 repair micro-loop.

## Definition of done
- [x] Guarantee levels declared/validated/hashed/recorded (AC56); unknown values
  rejected; changed claims re-key.
- [x] Content-addressed evidence store; stored bytes re-hash to the ledger's
  `evidence_hash` (AC57) — an audit gap found during design, closed in the same sprint.
- [x] Real Hypothesis property cell (derandomized) passes the determinism meta-check
  hermetically with `guarantee = "property"` recorded (AC58).
- [x] Judge gate: two-phase AND (det green + judge threshold), hash-chained judgments
  ledger, critique transcripts, verdict caching by `(cell_key, judge_hash)`, re-judge on
  judge change, det-red short-circuit (AC59–AC62).
- [x] Repair micro-loop: rejected unit repaired via critique, next attempt = next det
  round (R1 rejected → R2 green in test), budget + escalation failure record
  (AC63–AC64).
- [x] CLI: judged path auto-engaged by `judge.toml`; exit 0 iff two-phase green.
- [ ] Committed & pushed.

## Notable
- D18's satisfying discovery: §4.3's micro-loop needed **no loop machinery** — the loop
  body is literally `run_round`. Repair edits the unit, content addressing re-keys it,
  the frontier re-runs exactly what moved, the judge cache handles the rest. Attempts
  are ordinary numbered rounds in permanent history.
- The judge protocol is deliberately command-shaped so scripted judges prove the
  mechanics deterministically in CI; an LLM judge is the same protocol with a
  different command (and its identity is pinned by `judge_hash`, so a new prompt is a
  new judge — R-f answered).

## Next sprint (s8) candidates
T14 (sprint-loops Test-phase adapter — awaiting user's build-side decision), T7b
(contract enforcement), T8b (live Kani), T15b (durable self-host ledger → context
freeze), T12/T13 (mutation/fuzz), T3c (FS scoping).
