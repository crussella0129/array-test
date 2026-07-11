# array-test as the sprint-loops **Test** phase (T14)

[sprint-loops](https://github.com/crussella0129/sprint-loops) runs
**Research → Plan → Build → Test → Loop**, with the filesystem as the state machine. This
adapter makes array-test the *Test* phase: a sprint passes Test iff array-test certifies a
green root over the sprint's units, and the array-test state becomes the sprint-loops
project's durable, independently re-verifiable test memory.

array-test stays **consumer-agnostic** (decisions.md D11) — its core never references
sprint-loops. This adapter is the thin, optional shim on the sprint-loops side; it lives
here only so it can be developed and verified against the engine. Copy this directory into
your sprint-loops project (or the `array-test-fork`) to use it.

## The mapping

| sprint-loops | array-test |
|---|---|
| a sprint's deliverable code | units under `<project>/units/` (accreting across sprints) |
| a sprint's `test-plan.md` acceptance criteria | the units' cells (`[tests.unit\|direct\|closure\|e2e]`) |
| the **Test** phase verdict | a green array-test root (all cells Pass) |
| the sprint's durable test record | the persistent ledger + roots + evidence in `.array-test/state/` |
| "regression array travels backwards, confirmation at each step" | frontier reuse — each sprint's Test phase re-runs only the cells whose inputs changed |
| Loop back to Build on failure | the phase script exits non-zero; the round's Skipped/red cells name what to fix |

Because the state persists across sprints, sprint *N*'s Test phase is **incremental**: an
unchanged unit is reused (zero executions, byte-identical root), and a changed dependency
re-runs exactly the cells whose scope covers it. That is the founding schema's
down/out/backwards array, made economical by content addressing.

## Usage

```sh
# Whole-project convention: <project>/units/ + <project>/.array-test/state/
ARRAY_TEST_BIN=/path/to/array-test \
  adapters/sprint-loops/array-test-phase.sh --project . --sprint s7

# Or point at units/state explicitly:
adapters/sprint-loops/array-test-phase.sh --units ./units --state ./.array-test/state
```

- Exit `0` iff the Test phase is **green** (round all-Pass *and* the ledger re-verifies);
  `1` if **red** (loop back to Build); `2` on a usage/environment error.
- With `--sprint <name> --project <dir>`, a `sprints/<name>/test-record.md` is written with
  the verdict, the root, and the `array-test verify` command to re-check it with zero trust
  in the runner.
- The binary is found via `$ARRAY_TEST_BIN`, else `array-test` on `PATH`.

## Wiring it into the loop

In a sprint-loops sprint, invoke the phase where the loop's Test step runs — e.g. as the
command your sprint runner executes for Test, or as a CI step:

```sh
adapters/sprint-loops/array-test-phase.sh --project . --sprint "$SPRINT" || exit 1
```

A green exit advances the loop; a red exit keeps it in Build with a concrete, content-
addressed account of what failed and what was gated behind it.

## Note on `array-test-fork`

This adapter is meant to live on the sprint-loops side, in a fork (`array-test-fork`).
Creating that repository was **not possible from the session that built this** — its GitHub
access is scoped to the `array-test` repository only, so `create_repository` /
`fork_repository` for another repo return `403 Resource not accessible by integration`.
Create the fork under your account and drop this `adapters/sprint-loops/` directory in (or
re-run the build in a session whose GitHub scope includes it), and the Test phase is wired.
