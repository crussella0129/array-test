# s27 Research — does state roll back with the git tree?

The concern: an append-only, content-addressed state alongside version control (which is
not append-only) — does checking out an older commit leave array-test's state wrong?

## What the code guarantees
array_root (src/ledger.rs) folds the multiset {cell_key -> det_status} into a Merkle root.
cell_key (src/hash.rs, §2) derives from code_hash + scope deps + test_def + fixtures + seed
+ toolchain — all content, no round index, no wall-clock, no git metadata. So the latest
root is a pure function of the working tree. The ledger's append-only-ness is orthogonal:
it is the audit trail (§7.1), not the definition of "current state".

## Empirical confirmation (both levels)
- Content level: A -> edit -> B -> revert -> A. Root returns to A exactly; all cells reused.
- Git level: git init; commit A; run (root A); commit B (change); run (root B);
  `git checkout HEAD~1 -- units`; run -> root A with 3 reused (warm); then rm -rf state;
  run -> root A by re-execution (cold). B distinct throughout.

Conclusion: the tool tracks the git tree in both directions. Warm checkout = free reuse;
cold checkout (state not carried) = deterministic reproduction. Rollback never depends on
the cache surviving — it only gets faster when it does.

## Gap closed
Sound but untested: any future change threading a round number/timestamp into a cache key
or the root would break rollback silently. t17_rollback.rs now fails loudly if it does.
