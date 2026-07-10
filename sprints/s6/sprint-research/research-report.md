# s6 Research Report — Depth: the Scope Ladder and the Sandbox

## 1. Direction
User decision: this sprint hardens the deterministic core (T5b scope ladder, T3b
sandbox, R-h toolchain pinning) and must be complete before s7 takes the guarantee
tiers. T14 (sprint-loops adapter) is parked pending the user's build-side decision.

## 2. T5b — scope ladder semantics (→ D15)

### 2.1 What each scope's key includes
The scope determines which `code_hash`es enter the cell key — that *is* the scope's
meaning in a content-addressed world:

| Scope | dep hashes in key | re-runs when |
|---|---|---|
| `unit` | none (target only) | only the target (or its test) changes |
| `direct` | direct deps, topo order | target or a direct dep changes |
| `closure` | transitive closure, topo order | anything beneath it changes (s4 behavior) |
| `e2e` | **every unit** in the workspace | anything at all changes |

`e2e` including everything is the honest reading of "end-to-end": its validity really
does depend on the whole workspace, so its key must too. `unit` deliberately excludes
deps: mocking is the test author's business; the key records that this cell's validity
claims independence from them.

The scope itself is hashed into the key (a `SCOPE`-domain leaf), so the same test at two
scopes can never collide.

### 2.2 Declaration
`[tests.unit]`, `[tests.direct]`, `[tests.closure]`, `[tests.e2e]` tables in
`manifest.toml`, each a full TestSpec. Legacy `[test]` remains as sugar for
`[tests.closure]` (declaring both is a validation error). A unit with `[tests.e2e]` *is*
an entrypoint declaration (§1.4).

### 2.3 Fail-fast tier gating, with visible Skipped
Cheap scopes gate expensive ones (§5): rounds execute tier by tier
(unit → direct → closure → e2e). Once any completed tier contains a non-Pass, every cell
in higher tiers is recorded as **`Skipped`** — a new, ledger-visible `det_status` that is
not green and never cached (same D10 doctrine as Quarantined: skipping must be state,
not silence). Within a tier, a failure does not skip siblings (they're semantically
parallel). A reused Fail gates exactly like a fresh one — status gates, not freshness.

### 2.4 Per-scope envelopes
Default wall-clock: unit 10s, direct 30s, closure 60s, e2e 300s (`timeout_secs`
overrides). Memory caps are opt-in per test (`mem_limit_mb`, T3b) — imposing surprise
caps on existing cells would manufacture failures; the envelope's floor is the timeout.

## 3. T3b — sandbox (→ D16)

### 3.1 Memory: RLIMIT_AS via pre_exec
`mem_limit_mb` sets `RLIMIT_AS` in the child before exec; the whole process group
inherits it. Breach manifests as allocation failure/kill inside the cell → `Fail` (the
cell couldn't do its work within its declared envelope — that's a red cell, not a
timeout).

### 3.2 Network: namespace isolation, probed once, fail-closed
At first use the runner probes (once per process) whether it can create a network
namespace: `unshare(CLONE_NEWNET)` (root/CAP_SYS_ADMIN) or
`CLONE_NEWUSER|CLONE_NEWNET` (unprivileged). If the probe succeeds, **every** cell runs
in a fresh netns (loopback only, no routes out) using exactly the probed flags, and
pre_exec fails closed — a cell that cannot be isolated does not run. If the probe fails
(platform/container without the capability), cells run at env-hygiene level as before.
Either way the achieved level is **recorded per confirmation in the ledger**
(`isolation: env_only | net_isolated`) — D12's "the ledger records which guarantee level
applied", now real.

### 3.3 What stays open
Filesystem read scoping (bind-mount/chroot territory) remains the last R-g fragment —
recorded, not attempted this sprint. The meta-check continues to police what the sandbox
doesn't block.

## 4. R-h — toolchain pinning via `toolchain.lock` (→ D16)
Mechanism, not policy: if `<units-dir>/toolchain.lock` exists, its raw bytes hash
(TOOLCHAIN-domain leaf) into every cell key; changing the file re-keys the workspace.
What identifies a toolchain (e.g. `rustc -vV` output, a nix hash, an image digest) is
the consumer's policy to write into that file — the filesystem is the state machine.
Explicit `--toolchain-hash` still overrides; with neither, the honest "unpinned"
sentinel remains. This closes R-h at the mechanism level.

## 5. Recommendation
Land all three with full acceptance coverage before s7 (guarantee tiers T7/T8/T9/T10,
per user direction). Cell-key semantics change again this sprint (scope leaf, toolchain
source) — still pre-durable-ledger, so re-key is free (D9 precedent).
