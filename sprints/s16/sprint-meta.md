# Sprint s16 — Meta
- Sprint: 16
- Title: Test infrastructure — stop the silent CI skips (F11) + parser coverage (F12)
- Phase: loop
- Exit status: green
- Confidence: 1.0 (default 127 pass/3 ignored; --include-ignored 130 pass; clippy+fmt clean)

## Done
- F11: the three headline features (netns, read-only FS, real Hypothesis) now #[ignore]
  with reasons — reported as ignored, never falsely passed — and a privileged CI job
  runs them for real via `cargo test -- --ignored`. Verified live here with
  --include-ignored (this container is capable): all execute and pass.
- F12: co-located edge-case tests for tap::parse_libtest and hash::Hash::from_str.
- D27 records the scoping (F9/F13/F14 deferred as coverage-depth, not correctness-masking).

## Notable
F11 was the review's "single most impactful test-reliability issue": CI was *passing*
these features without *testing* them. The fix is the honesty doctrine again (D14/D19):
a skipped test must read as skipped, never as a pass.

## Next
s17 — the HashChainedLedger extraction (F1): the plan's single most impactful refactor,
dedups 4 copies and fixes the O(N^2) re-read bug in 3 sidecars. Byte layouts must stay
identical (rot guard is the witness).
