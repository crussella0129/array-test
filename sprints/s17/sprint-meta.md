# Sprint s17 — Meta
- Sprint: 17
- Title: Fix the O(N^2) sidecar appends (F1) + cache helper (F6) + typed errors (F4)
- Phase: loop
- Exit status: green
- Confidence: 1.0 (127 pass / 3 ignored; clippy+fmt clean; byte-preserving)

## Done
- F1 substance: mutation/fuzz appends are O(1) via open-once writers on a shared
  ChainState primitive. The generic HashChainedLedger<T> extraction is deferred (D28) as
  disproportionate machinery — the bug, not the DRY, was the value.
- F6: cache::read_cache<T> across round/judge/mutation/fuzz; fixes fuzz unwrap_or_default.
- F4: typed Spawn/Malformed/ChainBroken variants (mutation + fuzz).
- Strengthened: 2-entry multi-append chain-verify witness for the sidecars.

## Correction to the plan
F1's "3 of 4 have the bug" was 2 of 4 by s17 — judge.rs was fixed in s9. Recorded.

## Next
s18 (F2 enum-ify manifest) or s20 (security: F16 critique_ref traversal, F17 repair
sandboxing doc, F18 id validation). s20's F16/F18 are the higher-value of the remainder.
