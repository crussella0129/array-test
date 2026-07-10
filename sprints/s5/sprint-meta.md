# Sprint s5 — Meta

- **Sprint:** 5
- **Title:** TAP at the source; the self-hosting milestone
- **Phase:** loop (research, build, test complete)
- **Started:** 2026-07-10
- **Exit status:** green
- **Confidence:** 1.0 (77/77 tests; AC39–AC44 green on the first full run)

## Goal
Unblock and land self-hosting: a TAP adapter that produces evidence determinism at the
source (D14), then a round in which array-test certifies its own test suite.

## Definition of done
- [x] Research: adapter-vs-normalizer decision (normalization rejected on principle);
  wrapper spec; self-host env constraints.
- [x] T6 `src/tap.rs` + `array-test tap -- <cmd…>`: libtest → sorted, minimal,
  timing-free TAP 13; `ignored` → SKIP; synthetic `not ok` on silent nonzero exits.
- [x] T15 self-host integration test: our own t2 suite as a cell → Executed + Pass
  (not quarantined), green root, independent `verify` OK, round-2 reuse with identical
  root.
- [x] AC39–AC44 green (9 new tests; 77 total), clippy clean.
- [ ] Committed & pushed.

## Notable
- The self-host cell runs the **prebuilt libtest binary**, not `cargo test`: cargo holds
  the build-dir lock for its entire session, so an inner cargo would deadlock against
  the outer run — and the direct binary needs no PATH/HOME at all, making the cell
  *more* hermetic, not less. The plan's original `cargo test` shape was corrected during
  build; the research report's §4 command sketch is superseded by this (recorded in
  D14's note).
- The milestone completes the loop drawn in the original schema: the machine that
  produces confirmations is now itself a confirmed unit in its own array.

## Next sprint (s6) preview
Backlog order: T14 (sprint-loops Test-phase adapter — now with real certificates to
adapt), T5b (scope ladder), T3b (sandbox + R-h toolchain pinning), T15b (full self-host
workspace → first durable ledger → v1 context freeze), then the guarantee tiers
(T7 Hypothesis property tier, T8 Kani, T9/T10 judge gate + repair loop).
