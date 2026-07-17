# array-test

A system for agentic software test composition, implementation, and recording.

`array-test` models the regression suite as a **Merkle DAG of confirmations**: every test
is a content-addressed *cell* that only re-runs when something that affects it changes, and
a single green root hash certifies that all confirmations hold for exactly the current
code. This tames the exponential growth of regression complexity while staying
deterministic, code-based, and provable.

It is derived from a hand-drawn agentic testing schema (units built per sprint; an
integration/regression array that travels down, out, and backwards with a confirmation at
each step) and built using the [sprint-loops](https://github.com/crussella0129/sprint-loops)
protocol (Research → Plan → Build → Test → Loop; the filesystem is the state machine).

## Documentation
- [`docs/SCHEMA-ANALYSIS.md`](docs/SCHEMA-ANALYSIS.md) — reading of the original schema.
- [`docs/ARCHITECTURE.md`](docs/ARCHITECTURE.md) — the deterministic/provable design.

## Toolchain
**Rust** core engine (content addressing, DAG resolver, hermetic runner, ledger/Merkle
root, judge gate, CLI) + **Python (Hypothesis)** for property-based tests, connected by
**TAP** as a language-agnostic evidence contract (`decisions.md` D8).

## Sprint-loop state
- `decisions.md` — architectural decision log (D1–D34).
- `confidence.txt` — sprint-loop confidence throttle.
- `agent-tasks/` — active backlog + completion log.
- `sprints/sN/` — per-sprint research report + locked build/test plans + meta. All closed
  green; highlights below.

**Kernel (s0–s10) → v1.0.0.**
- `s0–s2` — design + testing-practice survey (D10); T1/T2 (content addressing, DAG);
  domain-separated hashing (D9), filesystem determinism, manifest validation.
- `s3–s5` — embedding contract (D11); hermetic cell runner (T3, D12); hash-chained
  ledger + array root (T4); round orchestrator + cache (T5, D13); CLI (T11); TAP evidence
  adapter (T6, D14); self-hosting (T15).
- `s6–s7` — scope ladder (T5b, D15); sandbox + recorded isolation (T3b, D16);
  `toolchain.lock`; guarantee tiers (T7 Hypothesis, T8 `proved`); Phase-J judge (T9, D17);
  repair micro-loop (T10, D18); evidence store.
- `s8–s10` — full-audit verifier (D19); `examples/quickstart`; review+refactor + the
  sidecar/value sequencing rule (D20); **v1.0.0** — the durable, rot-guarded self-host
  ledger froze the `array-test/v1/*` contexts (D21).

**Post-freeze extensions & the template (s11–s14).**
- `s11` — templatization: `docs/TEMPLATE.md`, the genesis ritual, CI, D22 (the repo is
  two templates in one).
- `s12` — the mutation tier (T12, D23) — the first post-freeze sidecar extension.
- `s13` — the fuzz tier (T13, D24), using the frozen `fixtures_hash` slot.
- `s14` — read-only-FS cells (T3c, mount_setattr, env-gated) + a contract-checker example.

**Refactoring pass (s15–s22), working through an external review (D26); every finding
substantiated or its premise corrected in the log.**
- `s15` — hygiene: LICENSE, `repr(u8)` on the frozen scope enum (F7), Cargo metadata,
  curated clippy lints.
- `s16` — test honesty: capability-gated tests are `#[ignore]` + a privileged CI job (F11,
  D27), not silent skips.
- `s17` — fixed the O(N²) sidecar appends via open-once writers on a shared chain
  primitive (F1, D28); a shared cache helper (F6); typed errors (F4).
- `s20` — security: `id` path-traversal validation (F18), defensive state-path
  containment (F16), trust-boundary docs (F17/F21) — D29.
- `s18` — typed the manifest scope keys so an unknown scope is a parse error (F2, D30);
  freeze-neutral.
- `s19`/`s22` — decomposed the four longest functions along their seams (F3, D31/D33);
  behavior-preserving.
- `s21` — single-pass evidence audit + brought this README current (D32).

**Proving the tiers (s23).**
- `s23` — made the `proved` guarantee tier **live** (T8b, D34): a committed CBMC
  bounded-model-checking cell (`examples/proved-cbmc/`) that verifies its claim over the
  whole input space, run for real in CI.

## Building & running
```
cargo build
cargo test      # 131 tests (+5 #[ignore], run in the privileged CI job); per-sprint
                # acceptance criteria live under sprints/*/sprint-plans/
cargo clippy --all-targets -- -D warnings

# Execute a regression round over a workspace of units; exit 0 iff green:
array-test run --units <units-dir> --state <state-dir> [--seed N] [--toolchain-hash blake3:HEX]

# Independently re-verify the ledger chain and latest round certificate:
array-test verify --state <state-dir>

# Wrap a libtest-style command in deterministic, timing-free TAP (the evidence
# adapter that makes wrapping `cargo test`-built binaries cache-stable):
array-test tap -- <command> [args...]
```

## Container image
A multi-stage [`Dockerfile`](Dockerfile) packages the release binary with the tools the
guarantee tiers need — CBMC (the `proved` tier) and Hypothesis (the `property` tier) — so
both are **live by default** instead of "provision it yourself". The image pins the
*evidence-producing environment* the way the engine pins the tested toolchain
(`toolchain.lock` → `toolchain_hash`): consume it **by digest** (`@sha256:…`), never by a
mutable tag. The `docker` CI job builds the image and proves it — quickstart and
proved-CBMC rounds green inside it, Hypothesis importable, and the sandbox live under
privilege (the ledger records `net_isolated`).

Two run modes (the isolation level is recorded honestly per confirmation either way):

```sh
# EnvOnly isolation: env hygiene + the determinism meta-check, no namespaces.
docker run --rm array-test:ci run --units /opt/array-test/examples/quickstart/units --state /tmp/s

# Full sandbox: per-cell network namespaces + read-only mounts need CAP_SYS_ADMIN.
docker run --rm --privileged array-test:ci run --units <units> --state <state>
```

Note the *image* is immutable (content-addressed layers); the running *container* is not —
add `--read-only` (with a writable `--tmpfs` for the state dir) if you want an immutable
runtime filesystem too.

## Distribution
Three channels, all first-class (D11):

1. **Container image** — published to GHCR by the `publish` CI job on every green push to
   `main`, tagged with the crate version and commit. **Consume it by digest** — the exact
   `docker pull ghcr.io/crussella0129/array-test@sha256:…` line is printed in each publish
   run's job summary. A digest pins the entire evidence-producing environment (binary +
   CBMC + Hypothesis + libc), the runner-side counterpart of `toolchain_hash`. The image
   also independently re-verifies any founding ledger with zero toolchain
   (`… verify --state /state`) — the `docker` CI job does exactly that against this repo's
   own genesis on every push, and `docs/TEMPLATE.md` documents the full container-path
   genesis ritual for template instances.
2. **Binary** — `cargo install --path .` from a clone (Rust ≥ 1.77). Not yet on crates.io.
3. **Library** — the `array_test` crate API (`run_round`, `full_audit`, ledgers, hashing);
   the CLI and the sprint-loops shim are both thin consumers of it.

## Notable design points
- Regression is a **Merkle DAG of confirmations**: content-addressed cells, frontier-only
  re-runs, a hash-chained ledger with a verifiable green root (`docs/ARCHITECTURE.md`).
- Confirmation is a **two-phase gate**: a deterministic, reproducible test phase (Phase D,
  what the Merkle root certifies) *and* an independent judge-agent review (Phase J, audited
  but not rooted). A judge rejection triggers a repair micro-loop scoped to the single unit,
  not the whole sprint (`docs/ARCHITECTURE.md` §4).
- Test authoring and evidence format follow
  [riteway](https://github.com/crussella0129/riteway)'s `given/should/actual/expected` +
  TAP conventions.

## Embedding
array-test is **library-first and consumer-agnostic** (D11). It is being built to power
the Test phase of the [sprint-loops](https://github.com/crussella0129/sprint-loops)
protocol, but the core never references sprint-loops — or any consumer. Integrate against
the stable outputs: the all-PASS green gate, `roots/R<k>.json` round certificates, the
independently re-verifiable hash-chained `confirmations.ndjson`, and hash-committed TAP
evidence. Anyone holding the ledger file can re-verify the chain and root with zero trust
in the runner.

For the sprint-loops protocol specifically, [`adapters/sprint-loops/`](adapters/sprint-loops/)
is a thin, optional Test-phase shim (T14): it runs a round over a sprint's units and gates
the sprint on a green, re-verified root, keeping the core agnostic (the adapter depends on
array-test, never the reverse).

## Using this repo as a template
The repo is two templates in one (D22): **the verification kernel** (frozen engine +
founding ledger — write units, get provable regression from commit one; all instances
speak the same v1 hash language) and **the method scaffold** (the sprint-loops records
that were the working memory which built it). See
[`docs/TEMPLATE.md`](docs/TEMPLATE.md) for instantiation and the **genesis ritual** —
each instance commits its own founding ledger, and CI's rot guard then protects its
history.

## Quickstart
See [`examples/quickstart/`](examples/quickstart/) for a runnable two-unit workspace
(guarded green by the test suite) demonstrating rounds, caching, the backwards arrow,
and full verification.

## Status
**v1.0.0.** The `array-test/v1/*` hash contexts and all byte layouts are **frozen**
(D21): the durable ledger at [`selfhost/state`](selfhost/) — array-test certifying
itself through its own CLI — commits to them permanently, and a rot-guard test audits
that history on every run. Post-freeze extension is by sidecar and by value (D20).

Sprints **s0–s23** all closed green — 131 tests (+5 `#[ignore]` capability tests run in a
privileged CI job; 136 with Hypothesis + CBMC provisioned), and the system is **self-hosting**: array-test runs its own test suite
as a cell (through the `tap` evidence adapter), passes its own determinism meta-check, and
certifies a green root over itself — then reuses that confirmation on the next round. Under
the hood: domain-separated
`code_hash`/`cell_key` content addressing (frozen `array-test/v1/...` contexts),
validated manifest/contract schemas, the integration DAG resolver, the hermetic cell
runner (cleared env + seed, evidence hashing, wall-clock envelope with process-group
kill, run-twice determinism meta-check → visible quarantine), the hash-chained
confirmation ledger with per-round root certificates, the cache-aware round
orchestrator (unchanged round ⇒ zero executions and a byte-identical root; a changed
dependency ⇒ exactly the keys whose scope covers it re-run), the **mutation tier**
(`array-test mutate`: a mutator command corrupts units, kill = red round, scores
memoized by the baseline root as detection-surface commitment — D23), the **fuzz tier**
(a fuzzer command over per-unit `fixtures/`, committed through the frozen `fixtures_hash`
slot — D24), the full scope ladder
(`[tests.unit|direct|closure|e2e]` with fail-fast tiers and ledger-visible Skipped),
the sandbox (memory caps, per-cell network namespaces where the host allows, isolation
level recorded per confirmation), `toolchain.lock` pinning, the guarantee tiers
(declared `example|property|proved` levels — with a real derandomized Hypothesis
property cell **and** a live CBMC bounded-model-checking `proved` cell, both passing the
meta-check; see `examples/proved-cbmc/`), the **two-phase confirmation gate** (`judge.toml`
→ an N-runs-vs-threshold judge command with hash-chained `judgments.ndjson`, critique
transcripts, and verdicts cached by `(cell_key, judge_hash)`), the **repair micro-loop**
(a rejected unit is patched from its critique and the next attempt is simply another
round — the frontier re-runs exactly what moved), a content-addressed re-hashable
evidence store, and the `run` / `verify` / `tap` CLI. See `agent-tasks/agent-tasks.md`.
