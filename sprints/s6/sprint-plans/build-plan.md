# s6 Build Plan — Finalized - DO NOT EDIT

## Tasks
1. **hash.rs** — `CellScope { Unit, Direct, Closure, E2e }`; scope leaf in the cell key
   (`SCOPE` context); `TOOLCHAIN` context.
2. **manifest.rs** — `[tests.<scope>]` map + legacy `[test]` (= closure; both → error);
   `mem_limit_mb` on TestSpec; scope-key validation.
3. **runner.rs** — `mem_limit_mb` on CellSpec → `RLIMIT_AS` in pre_exec; netns probe
   (once, `OnceLock`) + fail-closed unshare in pre_exec; `isolation_level()` public;
   `IsolationLevel { EnvOnly, NetIsolated }`.
4. **ledger.rs** — `DetStatus::Skipped`; `isolation` field in entries (canonical byte +
   serde), `append_entry` gains it; `append` defaults EnvOnly/false.
5. **round.rs** — per-scope cell planning (dep-hash table per D15), scope-tagged keys,
   tier-ordered execution with gate → Skipped, per-scope timeout defaults,
   `toolchain.lock` pickup (explicit hash > lock file > sentinel), report gains scope +
   Skipped kind.
6. **main.rs** — `--toolchain-hash` becomes override; report prints scope.
7. **Docs** — D15, D16; backlog (T5b, T3b, R-h → done; R-g narrowed to FS scoping);
   sprint files; README.

## Out of scope
Filesystem scoping; per-scope memory defaults; guarantee tiers (s7); T14.
