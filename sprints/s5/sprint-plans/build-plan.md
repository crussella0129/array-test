# s5 Build Plan — Finalized - DO NOT EDIT

## Tasks
1. **T6 — `src/tap.rs`.** `parse_libtest` (filter `test <name> ... ok|FAILED|ignored`
   lines), `render_tap` (sorted points, TAP 13, `# SKIP` for ignored, synthetic
   `not ok` when the inner command fails without a parsed failure), wrapper entry used
   by the CLI.
2. **CLI** — `array-test tap -- <command…>` subcommand; TAP on stdout, empty stderr,
   exit mirrors inner success.
3. **T15 — self-host integration test.** Workspace wrapping `cargo test --test
   t2_dag_resolver` through the tap adapter; assert Executed+Pass (not quarantined),
   green root, `verify` OK, second round reused with identical root.
4. **Docs.** D14 (determinism at the source, never normalization); backlog T6/T15 →
   completed; freeze-status note; README.

## Out of scope
T14 adapter; scope ladder; sandbox; property tier.
