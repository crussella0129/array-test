# s9 Build Plan — Finalized - DO NOT EDIT

Review sprint: apply findings F8–F16 (research report §3). No new features.

## Tasks
1. **F8** — `no-evidence` context for the skipped sentinel; `unpinned_toolchain` →
   `leaf(TOOLCHAIN, b"unpinned")`. (Last free re-key.)
2. **F9** — `Verdict::Quarantined { first: RunOutcome, second: RunOutcome }`; round
   stores both evidence blobs; ledger unchanged (first hash recorded).
3. **F10** — next round number from the ledger's max round, not the roots dir; remove
   `StatePaths::next_round`.
4. **F11** — `ConfirmationInput` + `Ledger::record()`; `append` kept as convenience;
   `append_entry` removed (round.rs migrates).
5. **F12** — judgments open-once: `JudgmentWriter` handle threaded through the round.
6. **F13** — `manifest.sprint` optional.
7. **F14** — `hex()` without per-byte alloc; exit-code sentinel comment; `cmd_run`
   judged path extracted to a function.
8. **F15** — ARCHITECTURE §7.4 "What a certificate attests" (cache trust model,
   re-run-from-empty-cache distrust protocol).
9. **F16** — audit note for ledger rounds without certificates.
10. **D20** — sequencing determination + sidecar/by-value extension doctrine +
    frozen-constants list (timeout defaults).

## Out of scope
T15b itself; any deferred tier.
