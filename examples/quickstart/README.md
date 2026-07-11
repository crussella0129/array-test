# Quickstart

A minimal two-unit workspace: `announcer` depends on `greeting`. Run a round:

```sh
cargo build
target/debug/array-test run --units examples/quickstart/units --state /tmp/quickstart-state
```

You'll see two cells execute (each run twice — the determinism meta-check), a green
root certificate written to `/tmp/quickstart-state/ledger/roots/R1.json`, and exit 0.

Run it again, unchanged:

```sh
target/debug/array-test run --units examples/quickstart/units --state /tmp/quickstart-state
```

Zero executions — both confirmations are reused from the cache, and R2's root is
byte-identical to R1's. Now edit `units/greeting/src/greeting.sh` (change the greeting
text so the tests still pass, e.g. keep the format) and run a third time: **both** cells
re-run — `greeting` because it changed, `announcer` because its closure-scope key
includes `greeting`'s code hash. That's the regression array's "backwards" arrow, with
no impact analysis anywhere: the keys carry the dependency structure.

Verify everything independently (chain, root certificates, judgments, evidence store):

```sh
target/debug/array-test verify --state /tmp/quickstart-state
```

## Adding the judge gate

Copy `judge.toml.example` to `units/judge.toml` and point `command` at any program
that reads `$ARRAY_TEST_UNIT_DIR` and prints a critique ending in `rating: <0-100>` —
an LLM CLI, a linter wrapper, a shell script. Cells must then pass **both** phases:
the deterministic tests AND the judge's threshold. A rejection with a `[repair]`
command configured triggers the repair micro-loop.
