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
