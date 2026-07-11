# s9 Research Report — Freeze-Readiness Review & the Sequencing Determination

## 1. The question
User: do T7b (contract enforcement), T8b (live Kani), T12/T13 (mutation/fuzz), and T3c
(FS scoping) need to land before T15b (durable ledger → v1 context freeze) so "1.0
proves itself fully" — or are they separable, with T15b simply next?

## 2. The determination: **they are separable; T15b is legitimately next**

The freeze (D9) locks *context strings and structural hashing rules* — the byte layouts
of `code_hash`, `cell_key`, `test_def`, ledger entries, roots, evidence framing. It
explicitly permits **adding new contexts**. So the question reduces to: does any
deferred tier need to change a frozen surface, or only to add alongside it?

Walking each one:

| Task | Needs a frozen-surface change? | Why not |
|---|---|---|
| T7b contract enforcement | **No** | Enforcement is a *command* (like everything here): a contract-checker invoked as/inside the cell's test command — already inside `test_def`. No new schema. |
| T8b live Kani (`proof_hash`) | **No** | The judgments ledger set the precedent: new evidence classes get their own hash-chained **sidecar** (`proofs.ndjson`, new contexts, keyed by `cell_key`) — additive, allowed post-freeze. |
| T12 mutation scores | **No** | Same sidecar pattern (`mutation.ndjson`), same reasoning. |
| T13 fuzz tier | **No** | Corpus is fixtures; `fixtures_hash` is already in the key. Pure consumer. |
| T3c FS scoping | **No** | A new `IsolationLevel` enum value is a new *byte value*, not a new byte layout — old entries verify unchanged. |

This is now doctrine (**D20**): *post-freeze extension happens by sidecar and by value,
never by relayout.* The judgments ledger wasn't just a feature — it was the extension
mechanism.

Two corollaries the review must enforce **before** freezing:
1. Anything in the current byte layouts we'd regret is due *now* (re-keying is free
   until T15b).
2. Per-scope timeout defaults (10/30/60/300) are hashed into `test_def` via the
   effective timeout — so those constants **freeze too**. Deliberate: the envelope is
   part of the definition. Recorded, not changed.

And a sharpening of what the deferred tiers are *for*: they prove 1.0's **claims**
(contracts enforced, proofs discharged, tests mutation-strong); T15b makes 1.0's
**promise** (keys stable forever). Shipping the promise first is not premature — it is
what makes the later tiers' results durable, because they'll be recorded against keys
that never move again.

## 3. Review findings (whole codebase, freeze lens first)

| # | Finding | Class | Action |
|---|---|---|---|
| F8 | Two sentinel hashes are domain-sloppy: skipped cells borrow the `EVIDENCE` domain for bytes that were never evidence; `unpinned_toolchain` uses un-domained `Hash::of`. No collision is *reachable* (framing constraints), but "unreachable" is a weaker property than "impossible by construction" — and this layout is about to freeze. | freeze | New `no-evidence` context for the skipped sentinel; unpinned toolchain → `leaf(TOOLCHAIN, "unpinned")`. Now, while re-keying is free. |
| F9 | Quarantine discards both transcripts: `Verdict::Quarantined` carries only hashes, and the store persists nothing — the one status whose *whole meaning* is "the two runs disagreed" keeps no evidence of either run. Debugging a quarantine currently means re-running it. | audit gap | `Quarantined` carries both `RunOutcome`s; the round stores both evidence blobs. Ledger schema unchanged (still records the first hash). |
| F10 | `next_round()` derives from the **roots directory** — but a crash between ledger-append and root-write would reuse the round number, silently merging two attempts' entries under one round. The ledger is the state machine; certificates are outputs. | correctness | Next round = max round in the *ledger* + 1. Remove the roots-scan. |
| F11 | `append_entry` is 8 positional args behind a clippy allow — a params struct wants to exist. | ergonomics | `ConfirmationInput` struct + `Ledger::record()`; `append` stays as a thin convenience. No byte changes. |
| F12 | `append_judgment` re-reads and re-verifies the whole judgments file per append — O(n²) rounds-with-judging. | perf debt | Open-once handle threaded through `judge_round`. |
| F13 | `manifest.sprint` is required — sprint provenance is our workflow, not every consumer's (D11 friction for standalone users). Not hashed (manifest is outside `code_hash`), so freely relaxable. | D11 polish | `#[serde(default)]`. |
| F14 | Cosmetics: `Hash::hex()` allocates a format! per byte; `exit_code.unwrap_or(i32::MIN)` sentinel is undocumented; `cmd_run`'s judged path is a 60-line nested block. | cosmetics | Fix all three. |
| F15 | **Trust-model gap in the docs, not the code:** the det cache is trusted-local — a tampered cache yields honestly-chained entries recording runs that never happened, and `full_audit` cannot catch it (nor should it pretend to). The distrust protocol is: re-run from an empty cache and byte-compare roots, which determinism makes possible. This is a *feature of the design* that was never written down. | docs | New §7.4 "What a certificate attests" in ARCHITECTURE. |
| F16 | `full_audit` doesn't note ledger rounds that lack certificates (the F10 crash shape). Not an integrity violation — but silence isn't a note either. | audit polish | Informational note for certificate-less rounds. |

## 4. Recommendation
Apply F8–F16 this sprint (F8/F10 are the freeze-gated ones); record D20; **then T15b
next sprint** — durable self-host ledger, v1 contexts frozen. The guarantee tiers
follow at leisure, extending by sidecar against stable keys.
