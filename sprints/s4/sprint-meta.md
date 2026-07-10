# Sprint s4 — Meta

- **Sprint:** 4
- **Title:** The first real R_k — frontier selection, cache, and the CLI
- **Phase:** loop (research, build, test complete)
- **Started:** 2026-07-10
- **Exit status:** green
- **Confidence:** 1.0 (68/68 tests green, all new ACs passed on first full run)

## Goal
Wire hash/dag/runner/ledger into an executable regression round (T5) with a standalone
CLI (T11), locking round semantics as D13.

## Definition of done
- [x] Research: round semantics locked (closure-scope v1 cells; Pass/Fail-only cache;
  per-round roots; reused-flagged entries; unpinned-toolchain sentinel R-h; self-hosting
  deferred to T15 with the cargo-timing rationale).
- [x] `manifest.toml` `[test]` section (command/env/timeout_secs) + validation.
- [x] `src/round.rs`: workspace, planning, cache, orchestration, `RoundReport`.
- [x] `src/main.rs`: `run` + `verify`, hand-rolled args, green-gate exit codes.
- [x] Ledger `append_entry` with in-hash `reused` flag; contexts `test-def`, `fixtures`.
- [x] AC29–AC38 green (11 new tests; 68 total), clippy clean.
- [ ] Committed & pushed.

## Notable
This sprint makes the schema's economics *observable*: round 2 of an unchanged workspace
executes nothing and reproduces the identical root byte-for-byte; changing the root
dependency of the a←b←c chain re-executes exactly the closure. The "backwards" arrow
needed no impact machinery at all — closure-scope keys made it emergent (D13.1).

## Next sprint (s5) preview
Candidates, in backlog order: T5b scope ladder, T6 TAP evidence adapter (unblocks T15
self-hosting), T3b sandbox, T14 sprint-loops adapter. T6→T15 is the highest-leverage
pair: self-hosting is the milestone where array-test certifies itself.
