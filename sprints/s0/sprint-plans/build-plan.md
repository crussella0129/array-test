# s0 Build Plan — Finalized - DO NOT EDIT

s0 is a **design sprint**: the "build" output is documentation + scaffolding, not engine
code. Engine construction begins in s1.

## Tasks (this sprint)
1. Write `docs/SCHEMA-ANALYSIS.md` — interpret the drawing (3 axes; wavefront semantics).
2. Write `docs/ARCHITECTURE.md` — Merkle-DAG-of-confirmations design; determinism;
   provability tiers; on-disk state machine; build order.
3. Seed `decisions.md` (D1–D5) and `agent-tasks/` backlog.
4. Establish `sprints/s0/` per the sprint-loops layout.
5. Commit + push to `claude/agentic-testing-schema-fsda10`.

## Explicitly out of scope (deferred to s1+)
- Any executable engine code (runner, ledger, DAG resolver).
- Language/toolchain selection (decided in s1 by determinism criteria — R-d).
- Property-based / formal tiers.

## Next sprint (s1) build targets
- **T1** content addressing + `manifest.toml`/`contract.toml` schemas + `code_hash`/
  `cell_key`.
- **T2** integration DAG resolver with forward (closure) + reverse (impact) traversal.
