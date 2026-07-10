# Sprint s9 — Meta

- **Sprint:** 9
- **Title:** Review + refactor: freeze-readiness audit and the sequencing determination
- **Phase:** loop (research, build, test complete)
- **Started:** 2026-07-10
- **Exit status:** green
- **Confidence:** 1.0 (107/107; zero new clippy allows — the one candidate was
  refactored away instead)

## Goal
User-requested review+refactor sprint, organized around one question: *what would we
regret freezing?* Plus the sequencing determination (T7b/T8b/T12/T13/T3c vs T15b).

## The determination (D20)
**Separable; T15b is next.** The freeze locks byte layouts and permits additive
contexts; every deferred tier extends by sidecar (the judgments-ledger pattern) or by
enum value — never by relayout. The tiers prove 1.0's *claims*; T15b makes its
*promise*, and stable keys are what make the tiers' later results durable.

## Definition of done
- [x] Review findings F8–F16 (research report §3) — all applied:
  - F8 sentinel domain hygiene (`no-evidence` context; TOOLCHAIN-domained sentinel) —
    the last free re-key.
  - F9 quarantine stores BOTH disagreeing transcripts (AC70).
  - F10 round numbers from the ledger, not the roots dir (AC71).
  - F11 `Ledger::record(ConfirmationInput)`; 8-arg method + its clippy allow gone.
  - F12 judgments open-once writer (O(n²) appends → O(n)).
  - F13 `manifest.sprint` optional (AC72).
  - F14 cosmetics (hex() allocs, exit-code sentinel doc, cmd_run extraction).
  - F15 ARCHITECTURE §7.4 trust model (integrity verified vs truthfulness reproduced).
  - F16 audit notes certificate-less rounds (AC71).
- [x] AC70–AC73 green (4 new tests; 107 total), clippy clean.
- [ ] Committed & pushed.

## Notable
The review's sharpest finding was F9: quarantine — the one status whose entire meaning
is "two runs disagreed" — was discarding both transcripts, making every quarantine
undebuggable without a re-run. Both runs' evidence is now content-addressed in the
store, and the audit covers it.

## Next sprint (s10)
**T15b**: the durable self-host ledger — and with it, the formal freeze of the
`array-test/v1/*` contexts.
