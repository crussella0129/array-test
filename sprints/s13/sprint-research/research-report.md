# s13 Research Report — T13: the Fuzz Tier, and Fixtures Become Real

## 1. The enabler: fixtures were a sentinel; the key slot was always there
`fixtures_hash` has been a frozen input slot of `cell_key` since s4 — filled with a
constant sentinel. T13 needs real fixtures (a fuzz corpus IS a fixture set), so the
slot gets its value mechanism: if `<unit>/fixtures/` exists, its content hashes
(same path-normalized, symlink-rejecting walk as `code_hash`; root node under the
existing FIXTURES context — role prefixes keep it distinct from the sentinel leaf);
absent dir ⇒ the sentinel, so **every existing workspace keeps its keys**. Value-level
change to a frozen slot: D20-legal, no relayout.

Cells read fixtures relatively (`fixtures/...` from their cwd). `code_hash` still
covers only `src/` + contract — fixtures re-key cells without pretending to be code.

## 2. The fuzzer is a command (fourth use of the pattern) → D24
`fuzz.toml`: `command` (receives ARRAY_TEST_UNIT_DIR, ARRAY_TEST_CORPUS_DIR =
`<unit>/fixtures/fuzz/`, ARRAY_TEST_SEED), optional `budget_secs` passed through as
ARRAY_TEST_FUZZ_BUDGET. Exit 0 = clean; **exit 65 = findings written into the corpus**.
Coverage-guided engines (cargo-fuzz/AFL), grammar fuzzers, or scripted probes all fit;
the engine only orchestrates.

The loop closes through content addressing, exactly as D10 predicted in s2: findings
land in the corpus → `fixtures_hash` moves → the unit's cells re-key → the next round
runs the tests against the grown corpus. No coupling between fuzzer and runner beyond
the filesystem.

Clean results are cacheable under `(code_hash, fuzzer_hash, fixtures_hash)` — honest
only because the fuzzer contract requires seed-determinism within a budget (documented;
a nondeterministic fuzzer wastes its own cache, nothing else breaks). Sidecar
`fuzz.ndjson` (contexts `fuzzer`, `fuzz-entry`, `fuzz-genesis`), audited like the rest.

## 3. Recommendation
`compute_fixtures_hash` in hash.rs; per-unit fixtures in plan_round; `src/fuzz.rs`
mirroring mutation.rs; `fuzz` CLI verb; scripted-fuzzer acceptance tests.
