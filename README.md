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

## Status
Sprints **s0–s6** all closed green — 90 tests, and the system is **self-hosting**:
array-test runs its own test suite as a cell (through the `tap` evidence adapter),
passes its own determinism meta-check, and certifies a green root over itself — then
reuses that confirmation on the next round. Under the hood: domain-separated
`code_hash`/`cell_key` content addressing (frozen `array-test/v1/...` contexts),
validated manifest/contract schemas, the integration DAG resolver, the hermetic cell
runner (cleared env + seed, evidence hashing, wall-clock envelope with process-group
kill, run-twice determinism meta-check → visible quarantine), the hash-chained
confirmation ledger with per-round root certificates, the cache-aware round
orchestrator (unchanged round ⇒ zero executions and a byte-identical root; a changed
dependency ⇒ exactly the keys whose scope covers it re-run), the full scope ladder
(`[tests.unit|direct|closure|e2e]` with fail-fast tiers and ledger-visible Skipped),
the sandbox (memory caps, per-cell network namespaces where the host allows, isolation
level recorded per confirmation), `toolchain.lock` pinning, and the `run` / `verify` /
`tap` CLI. Next up (s7): the guarantee tiers — Hypothesis property tier, Kani formal
tier, and the Phase-J judge gate + repair micro-loop. See `agent-tasks/agent-tasks.md`.
