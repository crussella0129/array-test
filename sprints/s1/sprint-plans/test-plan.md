# s1 Test Plan — Finalized - DO NOT EDIT

T1 and T2 are the first sprint with executable code, so this is the first *real* test
plan (s0 only had acceptance checks on docs).

## Acceptance checks — T1 (content addressing & schemas)
- [ ] **AC1** Two units with byte-identical `src/`+`contract.toml` produce the identical
  `code_hash`.
- [ ] **AC2** Any single-byte change to `src/` or `contract.toml` changes `code_hash`.
- [ ] **AC3** `cell_key` changes iff one of its declared inputs changes (target code,
  scope deps' code hashes, test_def, fixtures, seed, toolchain) — one property test per
  input.
- [ ] **AC4** Schema validation rejects a `manifest.toml` with a missing/malformed field.

## Acceptance checks — T2 (DAG resolver)
- [ ] **AC5** A cycle in `deps` is rejected with a clear error (never silently resolved).
- [ ] **AC6** Forward closure ("down") of a unit returns exactly its transitive deps, no
  more, no less, on a hand-built fixture graph.
- [ ] **AC7** Reverse closure ("backwards"/impact) of a unit returns exactly its
  transitive dependents, on the same fixture graph.
- [ ] **AC8** `dag.json` is deterministic — same input units produce byte-identical
  `dag.json` (ordering included).

## Authoring note
> **Amended:** originally said tests would migrate to riteway; per **D8**, the engine is
> Rust, so the target is a native Rust test harness, not riteway. Riteway remains an
> optional adapter for JS units only.

Per D6, these tests follow the `given/should/actual/expected` naming convention (Rust
`#[test]` functions with `given`/`should` in comments or test names) so that once T6's
native TAP emission lands, migrating is a mechanical rename, not a rewrite.

## Exit condition
All AC1–AC8 green → `R1` (the array's first real regression round) certified; root
written to `array/ledger/roots/R1.json`. Any failure → `sprints/s1/failure-report.md`,
confidence adjusted per the sprint-loops throttle.
