# s6 Test Plan — Finalized - DO NOT EDIT

AC1–AC44 stay green. New checks (unix):

## T5b scope ladder
- [ ] **AC45** A `unit`-scope cell does NOT re-key when a dependency changes (stays
  reused) while the same unit's `closure`-scope cell re-runs.
- [ ] **AC46** A `direct`-scope cell re-keys when a direct dep changes but NOT when a
  transitive (non-direct) dep changes.
- [ ] **AC47** An `e2e`-scope cell re-keys when ANY unit in the workspace changes.
- [ ] **AC48** The same test declared at two scopes yields two distinct cell keys
  (scope leaf).
- [ ] **AC49** Tier gating: a failing `unit`-tier cell ⇒ all higher-tier cells recorded
  `Skipped` (visible in ledger + report), round not green; same-tier siblings still run.
- [ ] **AC50** Skipped is never cached: once the gate lifts, previously-skipped cells
  execute.
- [ ] **AC51** Legacy `[test]` behaves as `[tests.closure]`; declaring both is rejected
  at manifest load.

## T3b sandbox
- [ ] **AC52** A cell exceeding its declared `mem_limit_mb` fails (not Pass).
- [ ] **AC53** When the netns probe succeeds, a cell sees only loopback in
  `/proc/net/dev` (conditional on host capability; skipped with a note otherwise).
- [ ] **AC54** Every ledger entry records the isolation level actually applied.

## R-h toolchain pinning
- [ ] **AC55** Adding/changing `toolchain.lock` re-keys every cell (all execute);
  explicit `--toolchain-hash` overrides the lock.

## Exit condition
AC1–AC55 green, clippy clean → s6 exits green.
