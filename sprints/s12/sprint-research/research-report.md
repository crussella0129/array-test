# s12 Research Report — The Mutation Tier (T12): Who Tests the Tests

## 1. Problem
A green array certifies "all tests pass" — which is only as strong as the tests
(s2 §2.5). Mutation testing measures that strength: corrupt the code, require the
suite to notice. Classically too expensive to run globally; our content addressing
changes the economics. This is also the **first post-freeze extension**, so it must
prove D20's doctrine: sidecar and value only, zero relayout.

## 2. Design (→ D23)

### 2.1 The mutator is a command (the judge/repair pattern, third time)
`<units-dir>/mutation.toml`:

```toml
command   = [...]   # receives ARRAY_TEST_UNIT_DIR (a scratch COPY of the unit),
                    # ARRAY_TEST_MUTANT_INDEX (0..mutants), ARRAY_TEST_SEED;
                    # edits the copy in place. Exit 0 = mutant produced;
                    # exit 64 = no mutant available at this index (skipped).
mutants   = 4       # mutants requested per unit
min_score = 100     # percent killed required to call the unit "mutation-strong"
```

Language awareness lives in the mutator (cargo-mutants, a sed script, an LLM —
anything); the engine only orchestrates and scores. `mutator_hash` pins
command+config, exactly like `judge_hash` (R-f logic).

### 2.2 Kill = a red round on the mutant workspace
For each mutant: copy the workspace, swap in the mutated unit, run a full round in a
scratch state. **Killed ⇔ the round is not green.** This is deliberately broader than
"the unit's own cells failed": a dependent's closure-scope cell catching the mutant is
the integration lattice doing its job, and it counts. (Baseline must be green before
mutation begins — on a red baseline, scores would be meaningless.)

Scratch rounds share one **cache** directory across mutants (content-addressed, so
sharing is safe by construction): unrelated cells hit cache after the first mutant,
and only mutant-covering keys re-run. The frontier economics apply *inside* the
mutation run too.

### 2.3 Score memoization: the cache key includes the detection surface
A unit's score depends not just on its own code but on **everything that could catch
the mutant** — every cell definition and dependency in the workspace. The honest
memoization key is therefore
`(unit code_hash, mutator_hash, baseline array root)` — the baseline root *is* a
commitment to the whole detection surface (`{cell_key → status}`), so any change to
any test, dep, seed, or toolchain re-mutates exactly when it should. Only dirty
frontier re-mutates; that is T12's whole point.

### 2.4 The sidecar (D20 proven)
Scores land in a new hash-chained `ledger/mutations.ndjson` (contexts
`array-test/v1/mutator`, `mutation-entry`, `mutation-genesis` — **additive**, legal
post-freeze), with per-unit entries: code_hash, mutator_hash, baseline root,
killed/survived counts, score, strong flag. `full_audit` gains a mutations-chain check
(a new check on a new surface — no frozen layout touched). Cache in
`state/mutation-cache/`.

### 2.5 CLI
A separate verb — `array-test mutate --units <dir> --state <dir> [--seed N]` — because
mutation is deliberately expensive; it should never ambush `run`. Exit 0 iff every
mutated unit is strong.

## 3. Recommendation
Build `src/mutation.rs` (mirroring judge.rs's shape), extend audit, add the CLI verb;
prove with scripted mutators: a content-checking unit kills all mutants (strong), an
assert-nothing unit lets them all survive (weak — the exact pathology mutation exists
to expose); memoization observed via mutator invocation markers.
