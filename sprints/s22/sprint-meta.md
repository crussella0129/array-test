# Sprint s22 — Meta
- Sprint: 22
- Title: Decompose run_cell (F3 follow-up; the function D31 deferred)
- Phase: loop
- Exit status: green
- Confidence: 1.0 (130 pass / 3 ignored; clippy -D warnings + fmt clean; behavior-preserving)

## Done
- run_cell (128 -> 55): build_cell_command / install_sandbox / wait_with_envelope.
- install_sandbox gains a # Safety doc (post-fork/pre-exec contract: async-signal-safe
  libc only, heap-free captures, fail-closed) — the F17/F21 trust-boundary doctrine applied
  to the last under-documented unsafe block.

## Coverage
- install_sandbox mem-cap branch: t3b non-ignored mem-cap test (normal CI test job).
- netns/mount branches: #[ignore]-gated t3b/t3c tests (privileged CI job).
Both CI jobs together exercise every branch. No byte/behavior change.

## Milestone
Completes the F1–F42 refactoring plan (substance done; stale premises corrected in the log;
maintainability-only extras deferred where documented). See D33.

## Next (all previously deferred side-quests)
T8b live Kani proof harness; T14 sprint-loops adapter; deeper docs/archival.
