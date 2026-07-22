# Agent Tasks — Active Backlog

Persistent across sprints (sprint-loops convention). Ordered by build dependency
(see ARCHITECTURE.md §9). Move finished items to `completed-tasks.md`.

**Toolchain (locked, D8):** Rust core engine (T1–T5, T9–T11); Python + Hypothesis for the
property tier (T7); TAP as the language-agnostic evidence contract (T6) — riteway is an
optional adapter for JS units, not a dependency of the core.

**Status:** refactoring plan (F1–F42, s15–s22), T8b (s23), T14 (s24), containerization
C1–C5 (s25/s26), and the rollback-soundness guard (s27/D38) complete. A full codebase
review (post-s27) filed the P0–P2 items below; the Kani handoff at the bottom still awaits
an authorized session.

## P0 — state integrity under adversity (one sprint; fix before calling it production-grade)
Findings from the post-s27 review, **verified live**, not speculative.
- [ ] **R1 — Concurrent-run lock.** Two simultaneous `run`s on one state dir corrupt it —
  reproduced: raced ledger appends → `verification FAILED: root certificate R1 has no
  ledger entries`. Detection works (verify exits 1); prevention doesn't exist. Fix: an
  exclusive advisory lock (`flock`) on `<state>/lock` held for `run`/`mutate`/`fuzz`;
  a second process fails fast with a clear message. Small; no frozen surfaces.
- [ ] **R2 — Crash durability (`fsync`).** `chained::append_ndjson_line` never syncs: an
  OS crash/power loss can lose acknowledged confirmations or tear the last ledger line —
  and a torn tail fails chain-verify with **no recovery path** (state bricked short of
  hand-editing). Fix: `sync_all` on ledger/sidecar appends (or minimally at round close +
  root write) **plus** an explicit `array-test repair --truncate-torn-tail` that recovers
  to the valid prefix *loudly* (honest recovery, never silent tolerance — D14).

## P1 — robustness & scale (one sprint each)
- [ ] **R3 — Evidence memory cap.** Both pipe-drain threads `read_to_end` into RAM; a
  runaway cell emitting GBs OOMs the runner (`runner.rs:382/387`). Fix: cap captured
  evidence (`evidence_limit_mb`, generous default) with an explicit truncation marker in
  the evidence bytes. Value-level, no relayout; determinism holds — both meta-check runs
  truncate identically.
- [ ] **R4 — Parallel cells within a tier (`--jobs N`).** Cells are hermetic and the code
  already says so (`round.rs:504`: "cells within a tier are semantically parallel") but
  execution is serial. Within-tier parallel execution is a large, low-risk speedup; tier
  gating stays sequential; ledger appends serialize behind the loop as today.
- [ ] **R5 — GC / growth policy.** `cache/`, `evidence/`, `critiques/` grow forever with
  no policy (the ledger grows by design — that part is correct). Fix: an `array-test gc`
  verb pruning cache entries + evidence unreferenced by the last N roots — evidence
  pruning is already audit-*legal* (missing evidence is a note, not a violation). At
  minimum, document the growth contract.

## P2 — polish
- [ ] **R6 — Contract honesty in docs.** `contract.toml` `[invariants]`/`[properties]` are
  schema-validated but engine-unenforced (by design — D17, Phase-J-audited declarations;
  the s14 example shows a unit self-enforcing). A newcomer will assume engine enforcement:
  make ARCHITECTURE §1.2 say "judge-facing declarations" in its first sentence, or promote
  a real contract-check tier.
- [ ] **R7 — Non-root container user.** No `USER` in the Dockerfile; the default path
  should run unprivileged (the sandbox mode can still be root). One line + a docs touch.

**Explicitly rejected by the review** (recorded so it isn't relitigated): no clap
migration (zero-dep CLI is deliberate; bad args already exit 2 cleanly); no generic
`HashChainedLedger<T>` (re-affirms D28); no engine awareness of git (rollback works
*because* the engine is VCS-blind — s27/D38 proved it).

## Containerization (complete: C1–C3 s25/D36, C4–C5 s26/D37)
- [x] **C1 — Multi-stage `Dockerfile`** (s25).
- [x] **C2 — Two documented run modes** (s25).
- [x] **C3 — `docker` CI job proving the shipped image** (s25).
- [x] **C4 — Publish by digest** (s26): `publish` CI job pushes to GHCR on green main
  pushes (crate-version + commit tags); the digest — the only pin that matters — lands in
  the job summary. README gained a "Distribution" section (image / binary / library).
  NOTE: first publish creates the GHCR package **private** by default; flip it to public
  in Settings → Packages if that's the intent (human step).
- [x] **C5 — Genesis-ritual parity** (s26): TEMPLATE.md documents the container-path
  ritual (builder image runs it — the selfhost cells need `cargo`; `--user` note for file
  ownership), and the `docker` CI job now has the runtime image independently re-verify
  this repo's committed founding ledger on every push (zero-toolchain audit, D22).

## Remaining
- [ ] **"Author your first unit" tutorial.** The schema is in ARCHITECTURE §1.1 and shown
  by `examples/`, but there's no dedicated walkthrough (manifest/contract/tests) for a
  newcomer.
- [ ] **Cross-platform proof.** 16/20 test files are `#![cfg(unix)]`; macOS runs degraded
  (EnvOnly), Windows is untested. Decide whether non-Linux is in scope; if so, prove it.
  (The container image is the sanctioned "run it anywhere" answer meanwhile.)
- [ ] **`array-test-fork` / sprint-loops wiring** — **user-owned** (being handled without
  AI): create the fork, drop in `adapters/sprint-loops/`. No agent action.
- [ ] **Kani (Rust proof path)** — see the HANDOFF plan below; blocked on egress scope in
  the sessions to date.

---

## HANDOFF — Kani provisioning plan. Read, execute (or re-file), then DELETE this section

**Delete this entire section once analyzed** — fold anything you aren't executing into the
"Remaining" list above as ordinary items. It is a transient note, not durable backlog.

**Goal.** Add a *native Rust* proof path to the `proved` tier: a committed
`#[kani::proof]` harness discharged by Kani, alongside (not replacing) the CBMC example.
The tier is prover-agnostic by design (D34) — this is an addition, not a migration.

**Why it hasn't happened.** Every session so far had GitHub egress scoped to
`crussella0129/array-test` only. `cargo kani setup` downloads its release bundle from the
`model-checking/kani` repo → 403. Verified, not assumed (D34 records the probes). Do NOT
route around the scope; run this in a session/host authorized for that repo — or use the
offline-bundle path in step 2.

**Plan (est. one sprint, purely additive — no frozen surface is touched):**
1. **Provision the driver:** `cargo install --locked kani-verifier` (crates.io — this part
   worked even in the scoped session).
2. **Provision the toolchain, one of:**
   a. Authorized host: `cargo kani setup` (downloads bundle + pinned nightly + CBMC).
   b. Offline: on any machine, fetch
      `kani-<ver>-x86_64-unknown-linux-gnu.tar.gz` from the Kani releases page, transfer
      it in, then `cargo kani setup --use-local-bundle <path>`.
   Sanity-check with `cargo kani --version` and a one-line proof crate.
3. **Build `examples/proved-kani/units/nibble-roundtrip-rs/`** mirroring
   `examples/proved-cbmc/` — same invariant (hex-nibble round-trip + hex-digit validity)
   so the two provers are comparable. A tiny cargo crate under `src/`, a
   `#[kani::proof]` harness with `kani::any::<u8>()`, a `run-proof.sh` wrapper, manifest
   with `guarantee = "proved"`.
   **Landmines the CBMC sprint already mapped — do not rediscover them:**
   - **`CARGO_TARGET_DIR` MUST point outside the unit dir** (e.g. a tmpdir). `code_hash`
     walks `src/**`; if cargo writes `target/` inside it, every run re-keys the cell —
     an infinite-re-key bug that will look like "caching is broken".
   - **Cells run with a cleared env** (D12). `cargo kani` needs `PATH` (with
     `~/.cargo/bin`), `HOME` (it reads `~/.kani`), and possibly
     `CARGO_HOME`/`RUSTUP_HOME` — declare them all in `[tests.unit.env]`.
   - **Determinism meta-check:** kani output carries timing/paths. The wrapper must
     discard raw output and emit fixed TAP lines only (the `run-proof.sh` pattern),
     or the run-twice check quarantines the cell.
   - Give the cell a generous `timeout_secs` (kani compiles before proving; 300+).
4. **Tests — `tests/t8c_proved_kani.rs`,** mirroring `t8b_proved.rs` exactly:
   one non-ignored plumbing check if anything engine-visible is new (likely nothing),
   plus `#[ignore = "requires kani; run via --ignored"]` + self-skip (D27) for: (a) the
   real proof passes and records `Guarantee::Proved`; (b) a falsified harness (assert
   something refutable) turns the round red — the proof must prove something.
5. **CI:** extend `privileged-tests` (or a new `kani` job): install driver + `cargo kani
   setup`, **cache `~/.kani` and the kani-verifier binary** (`actions/cache`; the setup
   download is ~hundreds of MB and slow), run `cargo test -- --ignored`. Confirm in the
   job logs that the kani tests *executed*, not skipped (the s23 log-check precedent).
6. **Records:** next free decision number (D39+ — D38/s27 were taken by the rollback
   guard) and next free sprint dir (s28+), research/plans/meta, backlog + README status
   updates. Optionally note a future "proof image" variant (builder stage + kani) under
   containerization.

**Definition of done:** kani tests pass live in CI (log-verified, not just green),
falsification case red, suite otherwise unchanged, zero frozen surfaces touched,
records written. The sprint-loops fork is NOT part of this — the user is handling it.
