# Sprint s12 — Meta

- **Sprint:** 12
- **Title:** T12 — the mutation tier: who tests the tests
- **Phase:** loop
- **Started:** 2026-07-11
- **Exit status:** green
- **Confidence:** 1.0 (113/113; one trivial compile fix in a test fixture, zero engine
  defects)

## Goal
The s2 survey's best synthesis, landed as the first post-freeze extension — proving
D20's sidecar-and-value doctrine in practice.

## Definition of done
- [x] `src/mutation.rs`: mutator-as-command protocol, workspace copies (symlink-
  rejecting), kill = red round over the mutant workspace, shared scratch cache,
  memoization keyed `(code_hash, mutator_hash, baseline_root)`, hash-chained
  `mutations.ndjson` sidecar.
- [x] Audit coverage for the sidecar; `array-test mutate` CLI verb (exit 0 iff all
  strong).
- [x] AC79–AC84 green (4 new tests; 113 total), fmt clean, clippy clean.
- [x] Zero frozen surfaces touched — additive contexts only (D20 proven by doing).
- [ ] Committed to dev; PR to main (new per-sprint workflow).

## Notable
The memoization key is the sprint's best idea: the baseline root already *is* a
commitment to the entire detection surface (every cell key and status), so
`(code_hash, mutator_hash, baseline_root)` re-mutates exactly when any test, dep,
seed, or toolchain changes — no bespoke dependency tracking needed. Content addressing
keeps paying for machinery we never had to build.

## Next candidates
T13 (fuzz tier — same sidecar pattern), T7b (contract enforcement), T8b (live Kani),
T3c (FS scoping), T14 (user's side-decision).
