# s7 Build Plan — Finalized - DO NOT EDIT

## Tasks
1. **Guarantee levels (T7/T8 schema).** `Guarantee { Example, Property, Proved }`;
   TestSpec `guarantee` (validated, default example), hashed into `test_def_hash`,
   recorded in ledger canonical bytes + serde.
2. **Evidence store.** `state/evidence/<evidence_hash>.tap` written for every executed
   cell (content-addressed; closes the discarded-evidence audit gap).
3. **T9 — `src/judge.rs`.** `judge.toml` load/validate; judge command protocol (env
   vars in, critique + `rating: N` out); N-runs/threshold/min_rating verdicts;
   hash-chained `judgments.ndjson` (+ genesis, own domain contexts) with
   `critique_ref` transcripts; judgment cache keyed `(cell_key, judge_hash)`;
   `run_with_judgment` orchestration (det round → judge det-Pass cells → verdicts).
4. **T10 — repair micro-loop.** `[repair]` config; on rejection run repair command
   (ARRAY_TEST_UNIT_DIR + ARRAY_TEST_CRITIQUE), then next attempt = next det round
   (frontier machinery re-runs what changed); budget; failure record
   `ledger/failures/R<k>-judgment.md` on exhaustion.
5. **T7 demo.** Real Hypothesis property cell (derandomized, TAP out), skip-if-absent.
6. **CLI.** `run` auto-uses the judged path when `judge.toml` exists; exit 0 iff
   det-green AND judge-green; judgment summary printed.
7. **Docs.** D17 (judge-as-command, judgment economics), D18 (repair = rounds);
   backlog/README/sprint files; T8b added.

## Out of scope
Running Kani (T8b); an actual LLM judge (the protocol is the deliverable; scripted
judges prove it); T14; T3c.
