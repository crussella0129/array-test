# s4 Research Report — Round Semantics for the First Real R_k

## 1. Problem
Wire hash/dag/runner/ledger into an actual regression round (T5) with a CLI surface
(T11). Several semantic decisions must be locked before code:

## 2. Decisions taken into the build (→ D13)

### 2.1 v1 cell scope = CLOSURE
The scope ladder (§1.4) will eventually give each unit UNIT/DIRECT/CLOSURE/E2E cells. v1
derives **one CLOSURE-scope cell per unit that declares a test** (`[test]` in
`manifest.toml`): its `cell_key` includes the transitive dep closure's `code_hash`es in
topological order. Rationale: closure scope is what makes the schema's "backwards" arrow
*emergent* — change a dependency and every transitive dependent's key changes, putting
exactly the impact set into the frontier with **no separate impact machinery**. The
reverse-dependency closure (`Dag::impact`) remains the *explanation* and the future
optimization; key inclusion is the *mechanism*. UNIT-scope (mocked deps) returns when the
ladder lands.

### 2.2 Cache policy: Pass and Fail are cacheable; Quarantined and TimedOut never
`cell_key → status` is forever (§2), so a deterministic Fail is as reusable as a Pass —
if nothing changed, the failure is still there; re-running it to "check" would be
superstition. Quarantined must never enter the cache (§6 already says so) — quarantine
means "we could not establish what this cell does". TimedOut is also uncacheable: wall
time depends on machine load, so a breach is evidence about the *host*, not only the
cell.

### 2.3 The root commits to the round's planned cells only
A naive "latest status per cell_key over the whole ledger" root would leak **stale
keys**: after a unit changes, its old cell_key's entry is still the ledger's latest for
that key and would pollute the commitment forever. The round root is computed from the
round's own entries — the planned, current array — while the full ledger remains the
append-only history. (History remembers everything; the certificate speaks only for now.)

### 2.4 Reused confirmations are ledger entries too, marked `reused`
Every round appends an entry per planned cell — fresh runs and cache hits alike — so each
round is self-contained in history ("real time / history" in the schema) and the root
derivation stays trivial. A `reused` flag (one byte inside the chained hash; additive,
pre-freeze) makes the audit trail say *this confirmation was inherited, here's the
evidence hash it inherited*. Added via `append_entry`; the existing `append` keeps its
signature.

### 2.5 Toolchain hash: explicit or an honest "unpinned" sentinel (gap R-h)
v1 does not auto-detect toolchains. The CLI accepts `--toolchain-hash`; absent that, a
fixed sentinel meaning *unpinned* enters the key. Recorded as gap **R-h**: a real
toolchain pinning story (hash of rustc/interpreter identity + lockfiles) is future work;
until then two machines with different toolchains can share cache entries they shouldn't.
Honest and visible beats silently wrong.

### 2.6 True self-hosting deferred (→ T15)
"array-test running its own cargo test suite as cells" stumbles on a real problem: cargo
prints wall-clock timings (`finished in 0.32s`), so evidence differs run-to-run and the
meta-check would (correctly!) quarantine every cell. The fix is a TAP-clean output path
(T6's adapter territory), not looser hashing. Deferred to **T15**; this sprint proves the
first real R_k on deterministic `sh`-based workspaces driven through the public API and
CLI — same machinery, honest evidence.

### 2.7 CLI: hand-rolled args, two verbs
`run` (execute a round; exit 0 iff green) and `verify` (re-verify chain + recompute the
latest round's root from the ledger; integrity only). Two verbs don't justify an argument-
parsing dependency in a crate whose value proposition includes a lean, pinnable dep tree.

## 3. Recommendation
Build `src/round.rs` (workspace, planning, cache, orchestration) + `src/main.rs`; extend
`manifest.toml` with `[test] command/env/timeout_secs`; contexts `test-def` + `fixtures`
join the v1 family. Backlog: T15 (self-host via TAP-clean output), R-h noted on T3b/T11.
