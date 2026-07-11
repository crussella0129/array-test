# s13 Build Plan — Finalized - DO NOT EDIT

1. hash.rs: `compute_fixtures_hash` (fixtures/ walk under FIXTURES context; sentinel
   when absent); contexts `fuzzer`, `fuzz-entry`, `fuzz-genesis` (additive).
2. round.rs: per-unit fixtures hash in `plan_round`.
3. `src/fuzz.rs`: config, `fuzzer_hash`, run-per-unit with corpus dir, exit-65
   findings protocol, clean-result cache `(code_hash, fuzzer_hash, fixtures_hash)`,
   chained `fuzz.ndjson`; audit coverage.
4. CLI `fuzz` verb (exit 0 iff no unit produced findings).
5. Tests AC85–AC89; D24; records; PR.
