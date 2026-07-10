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
