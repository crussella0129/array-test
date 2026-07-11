# s24 Build Plan
1. adapters/sprint-loops/array-test-phase.sh: --project/--sprint/--units/--state/--seed;
   run one round, re-verify, gate on green; write sprints/<sprint>/test-record.md; exit
   0 (green) / 1 (red) / 2 (usage). Binary via $ARRAY_TEST_BIN or PATH.
2. adapters/sprint-loops/README.md: integration model + mapping table + usage + the
   array-test-fork scope note.
3. tests/t14_sprint_loops_adapter.rs: green+record, broken->red, no-binary->usage error.
4. Validate live against examples/quickstart/units; verify full suite, clippy, fmt.
