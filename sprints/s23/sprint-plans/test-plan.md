# s23 Test Plan
- AC: `array-test run` over examples/proved-cbmc/units is ALL PASS with a proved cell.
- AC: the proved cell records Guarantee::Proved and DetStatus::Pass in the ledger.
- AC: a falsified harness (refutable assertion) makes the round red.
- AC: a `guarantee = "proved"` declaration records Guarantee::Proved without any prover
  (runs on every host).
- Honesty: the cbmc tests are #[ignore]+self-skip; the privileged CI job runs them for real.
- Gate: 131 pass / 5 ignored normally; 136 with cbmc; clippy -D warnings + fmt clean.
