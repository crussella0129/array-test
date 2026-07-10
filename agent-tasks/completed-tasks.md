# Completed Tasks

Append-only completion log (sprint-loops convention).

## Sprint s0
- [x] **S0-A — Read & analyze the hand-drawn schema.** → `docs/SCHEMA-ANALYSIS.md`
- [x] **S0-B — Design the deterministic/provable regression architecture.** →
  `docs/ARCHITECTURE.md`
- [x] **S0-C — Establish sprint-loops state** (decisions, confidence, agent-tasks,
  sprints/s0).

## Sprint s1
- [x] **T1 — Content addressing & schemas.** [Rust] `src/hash.rs` (`code_hash`,
  `cell_key`, domain-separated blake3 combination), `src/manifest.rs`, `src/contract.rs`.
  AC1-AC4 green.
- [x] **T2 — Integration DAG resolver.** [Rust, petgraph] `src/dag.rs` — cycle detection,
  forward closure ("down"), reverse/impact closure ("backwards"), deterministic
  `dag.json` serialization. AC5-AC8 green.
- [x] **Toolchain lock (D8).** Rust core + Python/Hypothesis property tier, resolving
  R-d.

## Sprint s2
- [x] **Testing-practice survey.** 10 topics mapped to the architecture with verdicts
  (D10); yielded T12 (frontier-scoped mutation testing), T13 (fuzz tier), and spec
  clauses for T3 (quarantine visibility, resource envelopes, coverage-as-metadata).
- [x] **F1 — Domain-separated hashing (D9).** Frozen `array-test/v1/...` derive_key
  contexts + 0x00/0x01 leaf/node role prefixes; fixed s1's hasher whose docs claimed
  separation it didn't implement.
- [x] **F2/F3/F4 — Filesystem determinism.** UTF-8 `/` path normalization, string sort,
  symlink + non-UTF-8 rejection, typed `CodeHashError`.
- [x] **F5 — Manifest load-time validation.** Empty id / self-dep / duplicate deps.
- [x] **F6 — `Dag::topo_order()`.** Deterministic deps-before-dependents order.
- [x] **F7 — Dependency bumps.** petgraph 0.8, thiserror 2, toml 1.

## Sprint s3
- [x] **T3 — Hermetic cell runner (v1 level, D12).** [Rust] `src/runner.rs`: cleared env
  + hygiene set + `ARRAY_TEST_SEED`, framed stdout/stderr/exit evidence hashing,
  wall-clock envelope with process-group kill, run-twice determinism meta-check →
  visible quarantine. Memory/network isolation deferred to T3b (R-g).
- [x] **T4 — Confirmation ledger + array root.** [Rust] `src/ledger.rs`: hash-chained
  append-only ndjson over canonical bytes, full-chain verification, reproducible array
  root over `{cell_key → det_status}`, `roots/R<k>.json` round certificates.
  `DetStatus` widened to Pass/Fail/Quarantined/TimedOut (D10 visibility).
- [x] **Embedding contract (D11).** Library-first; sprint-loops is consumer #1 via
  stable outputs, never a dependency.

## Sprint s4
- [x] **T5 — Frontier selection + cache.** [Rust] `src/round.rs`: workspace loading,
  closure-scope cell planning in topo order, Pass/Fail-only cache, cache-aware round
  execution, per-round root certificates, reused-flagged ledger entries (D13). The
  frontier economics are test-proven: unchanged round → 0 executions and an identical
  root; leaf change → 1 execution; root-dep change → full closure re-runs.
- [x] **T11 — CLI.** [Rust] `src/main.rs`: `array-test run` (exit 0 iff green) and
  `array-test verify` (chain + latest-root integrity, catches tampering). Hand-rolled
  args; consumer-agnostic per D11.
