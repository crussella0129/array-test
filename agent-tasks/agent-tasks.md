# Agent Tasks — Active Backlog

Persistent across sprints (sprint-loops convention). Ordered by build dependency
(see ARCHITECTURE.md §9). Move finished items to `completed-tasks.md`.

**Toolchain (locked, D8):** Rust core engine (T1–T5, T9–T11); Python + Hypothesis for the
property tier (T7); TAP as the language-agnostic evidence contract (T6) — riteway is an
optional adapter for JS units, not a dependency of the core.

## Engine (s7+)
- [ ] **T3c — Filesystem read scoping.** [Rust] The last R-g fragment: bind-mount/
  chroot-style scoping so a cell can only read its declared inputs. (Memory caps and
  network isolation landed s6, D16.)
- [ ] **T15b — Full self-hosting workspace.** Extend the T15 milestone (one suite,
  landed s5) to a committed `selfhost/` workspace covering every test suite as units
  with real deps, producing the first durable ledger — which formally freezes the v1
  hash contexts (D9, s5 research §5).


## Guarantees (s8+)
- [ ] **T7b — Contract tier.** [Rust] Enforce `contract.toml` pre/post/invariants per
  cell (the property tier landed s7; contracts are still declarations only).
- [ ] **T8b — Live Kani tier.** [Rust + Kani] Environment-gated: actually discharge
  designated critical-unit invariants with the model checker; attach `proof_hash`.
  (`guarantee = "proved"` schema landed s7.)
- [ ] **T12 — Frontier-scoped mutation testing.** [Rust + cargo-mutants] Mutation score
  as guarantee-level metadata per confirmation; only dirty units re-mutate — scores are
  memoized by `code_hash`, the same economics as the regression frontier. (D10;
  s2 research report §2.5.)
- [ ] **T13 — Fuzz tier (opt-in).** [Rust + cargo-fuzz] Coverage-guided fuzzing per
  unit; corpus is a content-addressed fixture set, so corpus growth re-keys cells
  naturally. (D10; s2 research report §2.6.)

## Surface (s5+)
- [ ] **T14 — sprint-loops Test-phase adapter.** Shim (sprint-loops side, or an adapter
  doc here) mapping array-test's stable outputs (`roots/R<k>.json`, ledger, green gate)
  onto sprint-loops artifacts (`test-report.md`/`failure-report.md`, phase exit).
  One-directional per D11 — array-test never learns sprint-loops exists.
