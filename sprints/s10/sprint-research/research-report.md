# s10 Research Report — The Durable Ledger and the Freeze (T15b)

## 1. Problem
D9's freeze triggers on the first durable committed ledger. The design obstacle: a
committed self-host workspace must be **machine-independent**, or its ledger rots on
every machine but the one that produced it. The s5 self-host cell finds a prebuilt
libtest binary at a machine-specific, build-hash-suffixed path — fine for an ephemeral
CI test, unusable in committed manifests.

## 2. Design (→ D21)

### 2.1 The units test the CLI, reached by *relative PATH*
Two observations make machine-independence achievable:
- `exec` resolves **relative PATH entries against the cwd**, and a cell's cwd is its
  unit dir. So `env.PATH = "../../../target/debug:/usr/bin:/bin"` reaches the freshly
  built `array-test` binary from any clone, with no absolute paths anywhere.
- Cell keys contain **no paths at all** (code, deps, test-def, fixtures, seed,
  toolchain) — and an *inner* array-test run over a generated fixture workspace uses
  fixed content, seed 0, and the unpinned sentinel, so even the inner root hash is
  machine-independent. Self-hosted cells can therefore emit fully deterministic
  evidence.

The three units exercise the system through its own front door:
- `selfhost.tap` (unit scope): the tap adapter normalizes noisy libtest output to exact
  expected TAP.
- `selfhost.run` (unit scope): an inner `array-test run` over a generated workspace —
  green round, cache reuse on the second round, byte-identical root across rounds.
- `selfhost.verify` (closure scope, deps on `selfhost.run`): an inner state passes
  `verify`, then a flipped ledger byte makes it fail.

Test scripts live as files in each unit's `src/` (inside `code_hash`), invoked as
`/bin/sh src/check.sh` — no TOML quoting gymnastics, and editing a script re-keys its
cell, as it must.

### 2.2 What "durable" commits, and what it promises
`selfhost/state/` is committed after two rounds (R1 all-executed, R2 all-reused,
identical roots): the ledger, both certificates, the cache, and the evidence store.
The commitment is **history**: the past is tamper-evident forever (a rot-guard test
runs `full_audit` over the committed state on every CI run). It does *not* promise
future rounds share keys — engine evolution appends new-keyed rounds to the same
chain, which append-only history tolerates by construction.

### 2.3 The freeze itself
With a durable ledger committing to `array-test/v1/*` hashes, D9's condition is met:
- **The v1 contexts and byte layouts are FROZEN** (declared in D21 and in
  `hash.rs`'s domain-module docs). Additive contexts remain legal; relayout is a
  re-key event requiring a v2 namespace (D20's sidecar-and-value doctrine governs
  extension).
- Frozen constants (from s9): per-scope timeout defaults 10/30/60/300s; role prefixes
  0x00/0x01; every canonical byte layout in `hash.rs`/`ledger.rs`/`judge.rs`/evidence
  framing.
- Version: **1.0.0** — the promise (stable keys) is what 1.0 means here; the deferred
  guarantee tiers extend against it (D20).

### 2.4 Honest caveats
- The committed `toolchain.lock` records this environment's rustc; other environments
  should regenerate it (new keys, appended rounds — history unbroken).
- Scripts assume a POSIX-ish Linux userland (GNU sed in `verify`'s tamper step);
  documented in `selfhost/README.md`.

## 3. Recommendation
Build the workspace; run two rounds; commit state; add the rot-guard; declare the
freeze (D21, hash.rs, README); bump to 1.0.0.
