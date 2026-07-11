# s1 Build Plan — Finalized - DO NOT EDIT

> **Amended (same sprint, before any T1/T2 code was written):** the original toolchain
> line below said "Node/TypeScript (per D6 / R-d)." That was superseded by **D8**
> (`decisions.md`) before any code existed under the old assumption — Rust core engine,
> Python/Hypothesis property tier. The toolchain line is corrected in place rather than
> left to silently disagree with the decisions log; see `sprint-research/research-report.md`
> addendum for the full comparison that produced D8.

Scope is deliberately narrow: only what the s1 research report recommends building now
(T1, T2). T6 (evidence adapter) and T9/T10 (judge gate + repair micro-loop) are documented
in ARCHITECTURE.md but **not** built this sprint — they carry open questions (R-e, R-f)
that don't block T1/T2 and shouldn't gate them.

## Tasks (this sprint)
1. **T1 — Content addressing & schemas.**
   - `manifest.toml` + `contract.toml` schema (ARCHITECTURE.md §1.1–§1.2).
   - `code_hash = H(src ‖ contract)`, `cell_key` per §2 — implemented, tested, documented.
   - Toolchain: **Rust** (per D8), `blake3` crate for content hashing over canonicalized
     input, `serde`/`toml` for schema parsing.
2. **T2 — Integration DAG resolver.**
   - **Rust**, `petgraph` for the graph representation.
   - Parse `deps` across all units into `dag.json`.
   - Cycle detection (fail loudly — the DAG in §1.3 is defined to be acyclic).
   - Forward traversal (dependency closure, "down") and reverse traversal (impact
     closure, "backwards"), per §1.3 and §3 step 2.

## Explicitly out of scope (deferred)
- Hermetic cell runner (T3), ledger (T4), frontier selection (T5) — need T1/T2 first.
- TAP evidence adapter (T6) — native Rust test harness emits TAP directly; no decision
  needed on capturing it from an external process.
- Property tier (T7) — Python + Hypothesis subprocess boundary; not needed until T3
  (runner) exists to invoke it.
- Judge gate / repair micro-loop (T9/T10) — needs R-e/R-f resolved first (judge model,
  prompt, retry-budget defaults).

## Next sprint (s2) build targets
- **T3** hermetic cell runner + determinism meta-check.
- **T4** confirmation ledger + Merkle root.
- Revisit **R-e/R-f** so T9/T10 can be scheduled.
