# Sprint s6 — Meta

- **Sprint:** 6
- **Title:** Depth: the scope ladder, the sandbox, and toolchain pinning
- **Phase:** loop (research, build, test complete)
- **Started:** 2026-07-10
- **Exit status:** green
- **Confidence:** 1.0 (90/90 tests; two fixture defects found and fixed in-sprint, zero
  engine defects)

## Goal
User-directed: harden the deterministic core (direction 2) completely before s7 takes
the guarantee tiers (direction 3). T14 parked pending the user's sprint-loops-side
decision.

## Definition of done
- [x] T5b scope ladder (D15): per-scope dep-hash semantics (unit/direct/closure/e2e),
  scope-tagged keys, `[tests.<scope>]` + legacy back-compat, fail-fast tiers with
  ledger-visible `Skipped`, per-scope timeout defaults.
- [x] T3b sandbox (D16): RLIMIT_AS memory caps; probed netns isolation (fail-closed
  when available — and it IS available in this environment: AC53 asserted cells see
  only loopback); isolation level recorded per confirmation in the chained ledger.
- [x] R-h closed at mechanism level: `--toolchain-hash` > `toolchain.lock` > sentinel.
- [x] AC45–AC55 green (12 new tests; 90 total), clippy clean.
- [ ] Committed & pushed.

## Notable
Both first-run failures were fixture bugs that taught something real:
1. `tail -c 1` keeps a ring buffer — it never holds the stream, so it's useless as a
   memory hog. The cap test now forces buffering via shell command substitution, with
   an uncapped control run proving the cap is what failed the capped cell.
2. Two byte-identical units shared a cell key and deduped through the cache — because
   `code_hash` covers src+contract, not the manifest. That is array-test working
   *correctly* (identical content is identical work); recorded as a design note in D15
   so nobody mistakes it for a bug later.

## Next sprint (s7) preview
User-directed: the guarantee tiers — T7 (Python/Hypothesis property tier over the TAP
boundary), T8 (Kani formal tier), T9 (Phase-J judge gate), T10 (repair micro-loop).
