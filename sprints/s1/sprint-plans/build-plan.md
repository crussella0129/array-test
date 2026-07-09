# s1 Build Plan — Finalized - DO NOT EDIT

Scope is deliberately narrow: only what the s1 research report recommends building now
(T1, T2). T6 (evidence adapter) and T9/T10 (judge gate + repair micro-loop) are documented
in ARCHITECTURE.md but **not** built this sprint — they carry open questions (R-e, R-f)
that don't block T1/T2 and shouldn't gate them.

## Tasks (this sprint)
1. **T1 — Content addressing & schemas.**
   - `manifest.toml` + `contract.toml` schema (ARCHITECTURE.md §1.1–§1.2).
   - `code_hash = H(src ‖ contract)`, `cell_key` per §2 — implemented, tested, documented.
   - Toolchain: Node/TypeScript (per D6 / R-d), content hash via a stable, well-audited
     hash (e.g. BLAKE3) over canonicalized input.
2. **T2 — Integration DAG resolver.**
   - Parse `deps` across all units into `dag.json`.
   - Cycle detection (fail loudly — the DAG in §1.3 is defined to be acyclic).
   - Forward traversal (dependency closure, "down") and reverse traversal (impact
     closure, "backwards"), per §1.3 and §3 step 2.

## Explicitly out of scope (deferred)
- Hermetic cell runner (T3), ledger (T4), frontier selection (T5) — need T1/T2 first.
- riteway/TAP evidence adapter (T6) — needs a decision on how TAP output is captured
  from a Node test process.
- Judge gate / repair micro-loop (T9/T10) — needs R-e/R-f resolved first (judge model,
  prompt, retry-budget defaults).

## Next sprint (s2) build targets
- **T3** hermetic cell runner + determinism meta-check.
- **T4** confirmation ledger + Merkle root.
- Revisit **R-e/R-f** so T9/T10 can be scheduled.
