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
- `decisions.md` — architectural decision log (D1–D8).
- `confidence.txt` — sprint-loop confidence throttle.
- `agent-tasks/` — active backlog + completion log.
- `sprints/s0/` — design sprint (closed, green): research report + locked build/test plans.
- `sprints/s1/` — closed, green: research (riteway investigation, two-phase confirmation
  gate, toolchain lock) + T1/T2 built and tested.
- `sprints/s2/` — closed, green: testing-practice survey (10 topics, adoption map in D10)
  + refactor: domain-separated hashing (D9), filesystem determinism, manifest validation,
  `topo_order()`.
- `sprints/s3/` — closed, green: embedding contract (D11), hermetic cell runner (T3,
  v1 isolation level per D12), hash-chained confirmation ledger + array root (T4).
- `sprints/s4/` — closed, green: round orchestrator + cache (T5, semantics in D13) and
  the standalone CLI (T11) — the first real `R_k`.
- `sprints/s5/` — closed, green: TAP evidence adapter (T6, principle in D14) and the
  self-hosting milestone (T15): array-test certifies its own test suite.
- `sprints/s6/` — closed, green: scope ladder (T5b, D15), sandbox with recorded
  isolation levels (T3b, D16), toolchain.lock pinning (R-h closed).
- `sprints/s7/` — closed, green: guarantee tiers — property tier with real Hypothesis
  (T7), `proved` schema (T8), Phase-J judge gate (T9, D17), repair micro-loop (T10,
  D18), content-addressed evidence store.
- `sprints/s8/` — closed, green: full-audit verifier (D19 — every root certificate,
  judgments chain, evidence store) + the committed `examples/quickstart` workspace.
- `sprints/s9/` — closed, green: review+refactor (findings F8–F16: sentinel hygiene,
  quarantine transparency, ledger-derived rounds, trust model §7.4) + the sequencing
  determination (D20): extension is by sidecar and by value; T15b next.
- `sprints/s10/` — closed, green: **v1.0.0** — the durable self-host ledger
  (`selfhost/state`, rot-guarded) froze the `array-test/v1/*` contexts (D21).

## Building & running
```
cargo build
cargo test      # AC1-AC38 (per-sprint test plans under sprints/*/sprint-plans/)
cargo clippy --all-targets

# Execute a regression round over a workspace of units; exit 0 iff green:
array-test run --units <units-dir> --state <state-dir> [--seed N] [--toolchain-hash blake3:HEX]

# Independently re-verify the ledger chain and latest round certificate:
array-test verify --state <state-dir>

# Wrap a libtest-style command in deterministic, timing-free TAP (the evidence
# adapter that makes wrapping `cargo test`-built binaries cache-stable):
array-test tap -- <command> [args...]
```

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

Sprints **s0–s10** all closed green — 109 tests, and the system is **self-hosting**:
array-test runs its own test suite as a cell (through the `tap` evidence adapter),
passes its own determinism meta-check, and certifies a green root over itself — then
reuses that confirmation on the next round. Under the hood: domain-separated
`code_hash`/`cell_key` content addressing (frozen `array-test/v1/...` contexts),
validated manifest/contract schemas, the integration DAG resolver, the hermetic cell
runner (cleared env + seed, evidence hashing, wall-clock envelope with process-group
kill, run-twice determinism meta-check → visible quarantine), the hash-chained
confirmation ledger with per-round root certificates, the cache-aware round
orchestrator (unchanged round ⇒ zero executions and a byte-identical root; a changed
dependency ⇒ exactly the keys whose scope covers it re-run), the **mutation tier**
(`array-test mutate`: a mutator command corrupts units, kill = red round, scores
memoized by the baseline root as detection-surface commitment — D23), the full scope ladder
(`[tests.unit|direct|closure|e2e]` with fail-fast tiers and ledger-visible Skipped),
the sandbox (memory caps, per-cell network namespaces where the host allows, isolation
level recorded per confirmation), `toolchain.lock` pinning, the guarantee tiers
(declared `example|property|proved` levels — with a real derandomized Hypothesis
property cell passing the meta-check), the **two-phase confirmation gate** (`judge.toml`
→ an N-runs-vs-threshold judge command with hash-chained `judgments.ndjson`, critique
transcripts, and verdicts cached by `(cell_key, judge_hash)`), the **repair micro-loop**
(a rejected unit is patched from its critique and the next attempt is simply another
round — the frontier re-runs exactly what moved), a content-addressed re-hashable
evidence store, and the `run` / `verify` / `tap` CLI. See `agent-tasks/agent-tasks.md`.
