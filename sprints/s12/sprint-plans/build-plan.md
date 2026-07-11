# s12 Build Plan — Finalized - DO NOT EDIT

1. **hash.rs** — additive contexts: `mutator`, `mutation-entry`, `mutation-genesis`
   (D20: sidecar extension; no frozen surface touched).
2. **`src/mutation.rs`** — `mutation.toml` load/validate; `mutator_hash`; workspace
   copy + unit swap; per-mutant scratch rounds with a shared cache; kill = red round;
   score memoization keyed `(code_hash, mutator_hash, baseline_root)`; hash-chained
   `mutations.ndjson` writer/reader; `run_mutation` orchestration (baseline must be
   green).
3. **audit.rs** — mutations-chain check + count.
4. **CLI** — `array-test mutate --units <dir> --state <dir> [--seed N]`; exit 0 iff
   all mutated units are strong.
5. **Docs** — D23; backlog (T12 done); README; sprint records. PR to main (new
   workflow).

## Out of scope
Language-aware mutators themselves (cargo-mutants etc. are consumers of the protocol);
T13/T7b/T8b/T3c/T14.
