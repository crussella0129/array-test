# Using this repo as a template

This repository is two templates layered in one (D22). Decide which layer(s) you want,
then follow the instantiation steps.

## The two layers

**Layer A — the verification kernel.** The frozen v1 engine (`src/`, the CLI), the
committed founding ledger (`selfhost/state`), and the audit machinery. Keep this layer
and your project gets deterministic, provable regression from its first commit: write
units under a `units/` dir (cells are commands — any language), run
`array-test run`, read the certificate. Because the v1 hash contexts are frozen (D21),
**every instance speaks the same hash language**: any v1 ledger produced anywhere is
verifiable by any v1 binary anywhere.

**Layer B — the method scaffold.** The sprint-loops working state that built this
project: `decisions.md`, `agent-tasks/`, `sprints/sN/` (research → plan → build → test
→ loop records), `confidence.txt`. These files were not documentation *about* the work
— they were the working memory that produced it. Keep this layer to inherit the
development method itself.

## What "self-verified" means here (and what it doesn't)

The kernel certifies its own behavior through its own machinery: the founding ledger
is tamper-evident, a rot-guard test re-audits it on every CI run, and §7.4's protocol
makes distrust cheap (re-run with an empty cache, byte-compare roots). That is
*self-certifying* — integrity verified, truthfulness reproducible, self-hosting green.
It is **not** a formal proof of correctness; the proved tier (T8b) remains future work.

## Instantiating

1. **Create from template.** Flip *Settings → Template repository* on GitHub (a human
   step), then "Use this template" — or fork/clone.
2. **Choose layers.**
   - Layer A only: delete `sprints/`, `agent-tasks/`, `decisions.md`,
     `confidence.txt` — or keep them as a worked example.
   - Layer B only: delete `src/`, `tests/`, `selfhost/`, `examples/`, `Cargo.*` and
     keep the scaffold for a different engine.
3. **The genesis ritual (Layer A).** Your instance should commit its *own* founding
   ledger rather than inheriting this repo's history:
   ```sh
   rm -rf selfhost/state
   rustc -vV > selfhost/units/toolchain.lock   # pin YOUR environment
   cargo build
   target/debug/array-test run --units selfhost/units --state selfhost/state  # R1
   target/debug/array-test run --units selfhost/units --state selfhost/state  # R2 (all reused)
   target/debug/array-test verify --state selfhost/state                      # VERIFIED
   git add selfhost/state && git commit -m "genesis: founding ledger"
   ```
   The rot-guard test (`tests/t15b_durable.rs`) now guards *your* history.
4. **Start your first sprint (Layer B).** Reset `decisions.md` to D1, open
   `sprints/s0/` with a research phase, and let `agent-tasks/` carry the backlog.
5. **Write real units.** Start from `examples/quickstart/`; add a `judge.toml` when
   you want the two-phase gate (see `examples/quickstart/judge.toml.example`).

## What must never change in an instance

The `array-test/v1/*` contexts and every canonical byte layout are frozen (D21).
Extend by **sidecar** (new hash-chained files keyed by `cell_key`, the judgments-ledger
pattern) and by **value** (new enum values) — never by relayout (D20). If you truly
need a new layout, that is a v2 namespace and a full re-key, honestly declared.
