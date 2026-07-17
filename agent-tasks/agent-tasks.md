# Agent Tasks — Active Backlog

Persistent across sprints (sprint-loops convention). Ordered by build dependency
(see ARCHITECTURE.md §9). Move finished items to `completed-tasks.md`.

**Toolchain (locked, D8):** Rust core engine (T1–T5, T9–T11); Python + Hypothesis for the
property tier (T7); TAP as the language-agnostic evidence contract (T6) — riteway is an
optional adapter for JS units, not a dependency of the core.

**Status:** refactoring plan (F1–F42, s15–s22), T8b (s23), T14 (s24), and containerization
C1–C3 + `--version` (s25) complete. Remaining below: C4/C5, the environmental blockers,
cross-platform, and the authoring tutorial.

## Containerization (C1–C3 landed s25/D36)
- [x] **C1 — Multi-stage `Dockerfile`.** rust:1-slim-trixie builder → debian:trixie-slim
  runtime with binary + cbmc + python3-hypothesis + the T14 shim + example workspaces.
- [x] **C2 — Two documented run modes.** README "Container image": plain = EnvOnly;
  `--privileged`/`--cap-add=SYS_ADMIN` = full sandbox; `--read-only` note for runtime
  immutability.
- [x] **C3 — A `docker` CI job.** Builds the image and proves the shipped artifact:
  quickstart + proved-CBMC rounds green inside it, Hypothesis importable, T14 shim works,
  and under `--privileged` the ledger records `net_isolated` (the non-theater witness).
- [ ] **C4 — Publish by digest.** Push to a registry (e.g. GHCR via a `packages: write`
  workflow on main) and document `docker run …@sha256:…`; extend the README "Container
  image" section into a full "Distribution" section (image / `cargo install` / library).
- [ ] **C5 (stretch) — Genesis-ritual parity.** Ensure the image path honors the template
  genesis ritual (D22) — a fresh instance can still commit its own founding ledger and let
  the rot guard protect it.

## Deferred / blocked (not software defects)
- [ ] **Kani (Rust proof path).** Add a native `#[kani::proof]` harness alongside the CBMC
  one. Blocked: `model-checking/kani`'s release bundle is outside this session's GitHub
  egress scope (D34). Needs a session/host authorized to reach it.
- [ ] **`array-test-fork`.** Create the sprint-loops-side fork and drop in
  `adapters/sprint-loops/`. Blocked: `create_repository`/`fork_repository` return 403 under
  this session's `array-test`-only scope (D35). Needs the user or a broader-scoped session.
- [ ] **Cross-platform proof.** 16/20 test files are `#![cfg(unix)]`; macOS runs degraded
  (EnvOnly), Windows is untested. Decide whether non-Linux is in scope; if so, prove it.
  (The container image is the sanctioned "run it anywhere" answer meanwhile.)

## Polish (in-scope, quick)
- [x] **`--version` / `-V` flag.** Landed s25: prints `array-test <crate version>`, exit 0;
  covered in t11_cli.
- [ ] **"Author your first unit" tutorial.** The schema is in ARCHITECTURE §1.1 and shown
  by `examples/`, but there's no dedicated walkthrough (manifest/contract/tests) for a
  newcomer.
