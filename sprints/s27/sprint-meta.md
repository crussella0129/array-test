# Sprint s27 — Meta
- Sprint: 27
- Title: Rollback-soundness guard + docs
- Phase: loop
- Exit status: green
- Confidence: 1.0 (137 pass / 5 ignored; clippy -D warnings + fmt clean; no engine change)

## Done
- Verified empirically (content-level AND a real git checkout): reverting the tree
  reproduces the earlier root — warm (cache, 0 re-exec) and cold (fresh state,
  deterministic re-exec). root_A -> root_B -> root_A byte-identical.
- tests/t17_rollback.rs guards it: warm revert (0 executed / all reused / ledger still
  audits clean with all rounds retained) + cold-state reproduction.
- Documented: ARCHITECTURE §7.1 "append-only ledger, content-addressed HEAD"; README
  design point.

## Finding
The property held by construction (array_root folds {cell_key -> det_status} with no
sequence/timestamp) but was unguarded. Now load-bearing. See D38.

## Next
Kani handoff (authorized session), authoring tutorial, cross-platform decision.
