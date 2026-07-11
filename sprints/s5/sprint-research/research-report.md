# s5 Research Report — TAP at the Source; the Self-Hosting Milestone

## 1. Problem
T15 (array-test certifying its own test suite) is blocked on evidence hygiene: cargo and
libtest print wall-clock timings (`finished in 0.32s`) and build noise, so raw evidence
differs run-to-run and the determinism meta-check — correctly — quarantines every cell.

## 2. The principle first (→ D14)
Two ways out, only one of them honest:

- **Normalize at the hasher** (strip "unstable" parts of evidence before hashing).
  REJECTED. This launders nondeterminism inside the trust boundary: the hash stops
  committing to what the cell actually emitted, and every normalization rule is a new
  place to hide a flake. s4 already named this trap: "the fix is clean evidence, not
  looser hashing."
- **Clean output at the source** (the cell's *command* emits deterministic TAP; evidence
  hashing stays byte-exact). ADOPTED. The adapter is part of the test definition —
  hashed into `test_def_hash` like any other input — and the runner keeps hashing
  exactly what the cell emitted, byte for byte.

The meta-check thus keeps its full power: if the adapter itself ever emits something
unstable, the cell quarantines, which is the correct outcome.

## 3. T6 design: `array-test tap -- <command…>`
A wrapper subcommand, usable inside any cell (and by non-Rust consumers wrapping
anything that emits libtest-style lines):

- Runs the inner command, capturing both streams; the wrapper's own stdout is the
  evidence-bearing stream and its stderr stays empty.
- Parses stdout lines of libtest shape (`test <name> ... ok|FAILED|ignored`); everything
  else — timings, `running N tests`, cargo's compile chatter — is dropped.
- **Sorts test points by name** and emits minimal TAP 13 (`TAP version 13`, plan,
  numbered points; `ignored` → `# SKIP`). Sorting buys determinism against
  multi-threaded interleavings; `--test-threads=1` in the inner command is still
  recommended (torn lines under concurrency are rare but possible).
- Exit code mirrors the inner command. If the inner command fails without any parsed
  `FAILED` line (e.g. a crash before tests, or unparseable output), a synthetic
  `not ok … - inner process exited nonzero` test point is appended — silence must never
  read as success.

## 4. T15 design: the self-host round
An integration test builds a workspace whose single unit wraps one of our own suites:

```
command = [<array-test bin>, "tap", "--", "cargo", "test", "--test", "t2_dag_resolver",
           "--", "--test-threads=1"]
```

- Declared env passes through exactly what cargo needs (`PATH`, `HOME`, `CARGO_HOME`/
  `RUSTUP_HOME` when set) — the hermetic runner's env_clear stays intact; the
  passthrough is *declared*, therefore inside `test_def_hash`.
- `CARGO_TARGET_DIR` points at the repo's target dir, so both meta-check runs are warm
  and compile chatter (consumed by the wrapper anyway) is minimal.
- Assertions: the cell is **Executed + Pass** (not quarantined — the whole point),
  round is green, `verify` passes, and a second round **reuses** the confirmation with
  an identical root: self-hosted frontier economics.

## 5. Freeze status (D9 follow-through)
The self-host ledger in CI is ephemeral (tempdir). The formal v1-context freeze
triggers on the first *durable* committed ledger — expected when T14 wires a real
project. Until then contexts remain additive-only in practice, frozen-on-commit by
policy.

## 6. Recommendation
Build `src/tap.rs` + the `tap` subcommand; prove determinism against injected timing
noise; land the self-host round as a standing integration test. Then s6 candidates: T14
(sprint-loops adapter, now with real output to adapt), T5b (scope ladder), T3b (sandbox).
