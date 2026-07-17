# s27 Build Plan
1. Verify empirically: content revert + real git checkout, warm and cold. (Done in research.)
2. tests/t17_rollback.rs: (a) change->revert => earlier root, executed()==0, reused()==2,
   full_audit clean with confirmations==6 (all rounds retained); (b) cold state dir over
   reverted content => earlier root by re-execution.
3. Docs: ARCHITECTURE §7.1 subsection; README design point.
4. Verify suite/clippy/fmt; D38 + sprints/s27; PR; merge on green.
