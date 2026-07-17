# Completed Tasks

Append-only completion log (sprint-loops convention).

## Sprint s0
- [x] **S0-A — Read & analyze the hand-drawn schema.** → `docs/SCHEMA-ANALYSIS.md`
- [x] **S0-B — Design the deterministic/provable regression architecture.** →
  `docs/ARCHITECTURE.md`
- [x] **S0-C — Establish sprint-loops state** (decisions, confidence, agent-tasks,
  sprints/s0).

## Sprint s1
- [x] **T1 — Content addressing & schemas.** [Rust] `src/hash.rs` (`code_hash`,
  `cell_key`, domain-separated blake3 combination), `src/manifest.rs`, `src/contract.rs`.
  AC1-AC4 green.
- [x] **T2 — Integration DAG resolver.** [Rust, petgraph] `src/dag.rs` — cycle detection,
  forward closure ("down"), reverse/impact closure ("backwards"), deterministic
  `dag.json` serialization. AC5-AC8 green.
- [x] **Toolchain lock (D8).** Rust core + Python/Hypothesis property tier, resolving
  R-d.

## Sprint s2
- [x] **Testing-practice survey.** 10 topics mapped to the architecture with verdicts
  (D10); yielded T12 (frontier-scoped mutation testing), T13 (fuzz tier), and spec
  clauses for T3 (quarantine visibility, resource envelopes, coverage-as-metadata).
- [x] **F1 — Domain-separated hashing (D9).** Frozen `array-test/v1/...` derive_key
  contexts + 0x00/0x01 leaf/node role prefixes; fixed s1's hasher whose docs claimed
  separation it didn't implement.
- [x] **F2/F3/F4 — Filesystem determinism.** UTF-8 `/` path normalization, string sort,
  symlink + non-UTF-8 rejection, typed `CodeHashError`.
- [x] **F5 — Manifest load-time validation.** Empty id / self-dep / duplicate deps.
- [x] **F6 — `Dag::topo_order()`.** Deterministic deps-before-dependents order.
- [x] **F7 — Dependency bumps.** petgraph 0.8, thiserror 2, toml 1.

## Sprint s3
- [x] **T3 — Hermetic cell runner (v1 level, D12).** [Rust] `src/runner.rs`: cleared env
  + hygiene set + `ARRAY_TEST_SEED`, framed stdout/stderr/exit evidence hashing,
  wall-clock envelope with process-group kill, run-twice determinism meta-check →
  visible quarantine. Memory/network isolation deferred to T3b (R-g).
- [x] **T4 — Confirmation ledger + array root.** [Rust] `src/ledger.rs`: hash-chained
  append-only ndjson over canonical bytes, full-chain verification, reproducible array
  root over `{cell_key → det_status}`, `roots/R<k>.json` round certificates.
  `DetStatus` widened to Pass/Fail/Quarantined/TimedOut (D10 visibility).
- [x] **Embedding contract (D11).** Library-first; sprint-loops is consumer #1 via
  stable outputs, never a dependency.

## Sprint s4
- [x] **T5 — Frontier selection + cache.** [Rust] `src/round.rs`: workspace loading,
  closure-scope cell planning in topo order, Pass/Fail-only cache, cache-aware round
  execution, per-round root certificates, reused-flagged ledger entries (D13). The
  frontier economics are test-proven: unchanged round → 0 executions and an identical
  root; leaf change → 1 execution; root-dep change → full closure re-runs.
- [x] **T11 — CLI.** [Rust] `src/main.rs`: `array-test run` (exit 0 iff green) and
  `array-test verify` (chain + latest-root integrity, catches tampering). Hand-rolled
  args; consumer-agnostic per D11.

## Sprint s5
- [x] **T6 — TAP evidence adapter (D14).** [Rust] `src/tap.rs` + `array-test tap -- <cmd>`:
  libtest output → minimal sorted timing-free TAP 13; ignored → SKIP; silent nonzero
  exits synthesize a `not ok`. Determinism produced at the source — evidence hashing
  stays byte-exact.
- [x] **T15 — Self-hosting milestone.** array-test runs its own t2_dag_resolver suite
  as a cell (prebuilt libtest binary through the tap adapter), passes the meta-check
  un-quarantined, certifies a green root over itself, survives `verify`, and reuses the
  confirmation on round 2 with an identical root. Full-workspace version → T15b.

## Sprint s6
- [x] **T5b — Scope ladder (D15).** [Rust] `[tests.unit|direct|closure|e2e]`;
  scope-tagged cell keys where the scope decides the dep-hash set (unit none / direct /
  closure / e2e = whole workspace); fail-fast tiers with ledger-visible `Skipped`
  (never cached, not green, siblings unaffected); per-scope timeout defaults.
- [x] **T3b — Sandbox (D16, partial by design).** [Rust] `mem_limit_mb` → RLIMIT_AS;
  one-time netns capability probe, fresh namespace per cell fail-closed when available;
  isolation level recorded per confirmation in the chained ledger. FS scoping → T3c.
- [x] **R-h — Toolchain pinning mechanism (D16).** explicit `--toolchain-hash` >
  `toolchain.lock` bytes > unpinned sentinel; lock changes re-key the workspace.

## Sprint s7
- [x] **T7 — Property tier + guarantee levels (D17).** `guarantee =
  example|property|proved` validated, hashed into test_def, recorded per confirmation;
  real Hypothesis property cell (derandomized) passes the meta-check hermetically.
- [x] **T8 — `proved` schema.** Recorded + audited; live Kani → T8b.
- [x] **T9 — Phase-J judge gate (D17).** `judge.toml` command protocol (critique +
  `rating: N`), N runs vs threshold, judge_hash identity pinning, hash-chained
  `judgments.ndjson` + critique transcripts, verdicts cached by (cell_key, judge_hash),
  det-red short-circuit, two-phase AND gate in report + CLI exit.
- [x] **T10 — Repair micro-loop (D18).** Repair command driven by the critique; each
  attempt is an ordinary det round (frontier re-runs exactly what the repair touched);
  budget + escalation failure record. Converges in tests; escalates with audit refs.
- [x] **Evidence store.** Executed cells persist exact framed evidence bytes,
  content-addressed and re-hashable against the ledger.

## Sprint s8
- [x] **T16 — Full-audit verifier (D19).** `audit::full_audit` as a library:
  confirmations chain, every root certificate, judgments chain, evidence store;
  problems vs notes strictly separated; CLI `verify` rewired; tamper detection proven
  for each surface.
- [x] **Quickstart example.** `examples/quickstart/` committed (dep edge, two scopes,
  README walkthrough, judge.toml.example), guarded green by an integration test.

## Sprint s11 (templatization)
- [x] **D22 — the repo as a two-layer template.** Kernel (frozen engine + founding
  ledger; instances share the v1 hash language) + method scaffold (the sprint-loops
  working memory). `docs/TEMPLATE.md` with the genesis ritual; CI workflow keeping the
  rot guard live on every push; fmt hygiene pass.

## Sprint s13 (fuzz tier + fixtures)
- [x] **T13 — Fuzz tier (D24).** Fixtures fill the frozen `fixtures_hash` slot (no
  fixture files ⇒ sentinel; pre-T13 keys preserved); fuzzer-as-command with exit-65
  findings written into `fixtures/fuzz/`; the loop closes through content addressing
  (findings → new keys → next round tests the grown corpus); chained `fuzz.ndjson` +
  audit; clean-result cache; `fuzz` CLI verb.

## Sprint s12 (mutation tier)
- [x] **T12 — Frontier-scoped mutation testing (D23).** Mutator-as-command; kill = red
  round; memoized by (code_hash, mutator_hash, baseline_root) — the baseline root as
  detection-surface commitment; hash-chained mutations.ndjson sidecar + audit; `mutate`
  CLI verb. First post-freeze extension; zero frozen surfaces touched.

## Sprint s10 (v1.0.0)
- [x] **T15b — The durable ledger + v1 freeze (D21).** Machine-independent `selfhost/`
  workspace (relative-PATH CLI cells, scripts in src/); founding rounds R1/R2 committed
  with byte-identical roots; rot-guard audit test; contexts FROZEN; version 1.0.0.

## Sprint s9 (review + refactor)
- [x] **Sequencing determination (D20).** T7b/T8b/T12/T13/T3c are separable from T15b:
  extension is by sidecar and by value, never by relayout. T15b is next.
- [x] **F8–F16 applied.** Sentinel domain hygiene (last free re-key); quarantine stores
  both transcripts; ledger-derived round numbers; `Ledger::record` struct API;
  judgments open-once; `manifest.sprint` optional; cosmetics; §7.4 trust model; audit
  notes certificate-less rounds.

## Sprint s14 (read-only FS + contracts)
- [x] **T3c — Filesystem read scoping.** Env-gated recursively read-only mount via
  mount_setattr (D25); fail-closed. Completes R-g isolation. (#[ignore] + privileged CI.)
- [x] **T7b — Contract-checker example.** A committed unit that enforces contract.toml.

## Sprints s15–s22 (refactoring pass, external review D26; F1–F42)
- [x] **s15–s17** — hygiene/repr(u8)/metadata/lints (D26); #[ignore]+privileged CI honesty
  (F11, D27); O(N^2) sidecar appends fixed via open-once writers (F1, D28); cache helper
  (F6); typed errors (F4).
- [x] **s20/s18/s19/s21/s22** — security: id validation + containment + trust docs (D29);
  typed manifest scope keys (F2, D30); decomposed the four longest functions (F3, D31/D33);
  single-pass evidence audit + README currency (D32).

## Sprint s23 (T8b — proved tier live)
- [x] **T8b — Live proof tier.** CBMC (Kani's engine) discharges an all-inputs proof over a
  committed `examples/proved-cbmc/` unit; guarantee=proved recorded; falsification test;
  runs live in the privileged CI job (D34). Kani Rust path deferred (its bundle host is
  outside session GitHub scope).

## Sprint s24 (T14 — sprint-loops adapter)
- [x] **T14 — sprint-loops Test-phase adapter.** `adapters/sprint-loops/array-test-phase.sh`
  + README; gates a sprint on a green, re-verified root; per-sprint test-record.md; core
  stays agnostic (D11/D35). array-test-fork creation blocked by session GitHub scope.
