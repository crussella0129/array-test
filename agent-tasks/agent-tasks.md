# Agent Tasks — Active Backlog

Persistent across sprints (sprint-loops convention). Ordered by build dependency
(see ARCHITECTURE.md §9). Move finished items to `completed-tasks.md`.

**Toolchain (locked, D8):** Rust core engine (T1–T5, T9–T11); Python + Hypothesis for the
property tier (T7); TAP as the language-agnostic evidence contract (T6) — riteway is an
optional adapter for JS units, not a dependency of the core.

**Status:** the full external refactoring plan (F1–F42, s15–s22), T8b (s23), and T14 (s24)
are complete and merged to `main`. 134 tests / 5 ignored (139 with Hypothesis + CBMC).
The next planned track is **containerization** (below).

## Containerization (s25 — proposed next)
Rationale: array-test is Linux-first (the netns / `mount_setattr` / `RLIMIT_AS` sandbox is
Linux-only) and the strongest tiers need provisioned tools (CBMC, Hypothesis). An image
makes the Linux environment guaranteed, bakes in those tools so `proved`/`property` are
live by default, and pins the *evidence-producing* environment by digest — the same
content-addressing discipline the engine applies to the tested toolchain (`toolchain.lock`
→ `toolchain_hash`). Ship it as **one distribution channel among several**, never the only
one: D11 keeps the binary + crate + library API first-class.

- [ ] **C1 — Multi-stage `Dockerfile`.** Builder stage (Rust toolchain, `cargo build
  --release`) → lean runtime stage carrying just the binary + `cbmc` + `python3` +
  `hypothesis`. No Rust toolchain in the runtime image. Pin the base image by digest.
- [ ] **C2 — Two documented run modes.** Plain `docker run` = EnvOnly isolation (honestly
  recorded); `--privileged` / `--cap-add=SYS_ADMIN` (+ userns) = full sandbox (netns +
  read-only mounts). The CI `privileged-tests` job already proves the sandbox works inside
  a privileged container — mirror its flags in the docs. Note that runtime immutability
  additionally needs `--read-only`; the *image* is immutable, the *container* is not by
  default.
- [ ] **C3 — A `docker` CI job.** Build the image and run the full suite
  (`cargo test -- --include-ignored`, or the packaged binary's self-host round) *inside*
  it, proving the sandbox + CBMC + Hypothesis all function in the image — not theater.
- [ ] **C4 — Publish by digest + README section.** Document `docker run …@sha256:…` (not
  `:latest`) so an instance pins an exact evidence-producing environment; add a
  "Distribution" section covering image / `cargo install` / library.
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

## Polish (in-scope, quick)
- [ ] **`--version` / `-V` flag.** Currently errors ("expected a subcommand"); `--help`
  works. One-line fix in `main.rs`; a convention users expect.
- [ ] **"Author your first unit" tutorial.** The schema is in ARCHITECTURE §1.1 and shown
  by `examples/`, but there's no dedicated walkthrough (manifest/contract/tests) for a
  newcomer.

---

## HANDOFF — read, then delete this section

**Delete this entire "HANDOFF" section once you have read and acted on it** (it is a
transient note to the next agent, not part of the durable backlog). Fold anything still
outstanding into the task lists above before deleting.

**Where the project is.** array-test is a working, self-hosting, deterministic/provable
regression engine at v1.0.0 with frozen `array-test/v1/*` hash contexts. It is usable now
as a Linux CLI, a Rust library, the Test phase of sprint-loops (via the adapter), and a
GitHub template. Everything the user asked for through this arc is done and merged: the
whole F1–F42 refactoring plan, the live `proved` tier (CBMC), and the sprint-loops adapter.
Decisions run through D35; sprints through s24.

**Direction.** The next deliberate step is **containerization** (C1–C5) — motivated by the
usability assessment: the tool is Linux-first and its strongest tiers need provisioned
tools, so an image that guarantees Linux + bakes in CBMC/Hypothesis + pins the environment
by digest turns "provision it yourself" into "live by default," and is philosophically on
brand for a content-addressed determinism tool. Treat the image as an *additional*
distribution channel, not a replacement — D11 keeps the binary/crate/library first-class.

**Watch-outs for whoever picks this up.**
- The sandbox needs elevated caps; document the two run modes (C2) or users silently get
  EnvOnly. CI already proves the sandbox works in a privileged container — copy those flags.
- Don't touch frozen surfaces (D20/D21): extend by sidecar and by value, never relayout.
  The rot guard (`t15b_durable`) is the tripwire.
- Keep the honesty doctrine (D14/D19/D27): a not-run capability must read as *ignored*,
  never *passed*. Any new gated feature gets `#[ignore]` + self-skip + a real CI job.
- Two blockers are environmental, not code: Kani's repo and repo-creation are outside this
  session's GitHub scope. Don't try to route around them — hand them to a scoped session.

**Suggested first move.** Open s25 for C1–C3 (Dockerfile + run modes + a `docker` CI job
that runs `--include-ignored` inside the image), through the usual dev → PR → main flow with
CI green before merge.
