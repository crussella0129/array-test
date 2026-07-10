# s2 Build Plan — Finalized - DO NOT EDIT

Refactor sprint driven by the s2 research report (§3 findings F1–F7). No new features —
the observable surface is hardened, not extended, except one small forward API (F6).

## Tasks
1. **F1 — Domain-separated hashing.** Rewrite `hash.rs` around BLAKE3 `derive_key`
   contexts (`array-test/v1/...`, frozen). Separate `Hash::leaf(context, bytes)` and
   `Hash::node(context, parts)` constructors; retire the untagged `combine`. Every
   production hash names its domain at the call site.
2. **F2/F3/F4 — Filesystem determinism.** Normalize relative paths to `/`-joined UTF-8;
   reject non-UTF-8 names and symlinks with explicit errors; sort by normalized string.
   `compute_code_hash` returns a typed error (`CodeHashError`) instead of bare `io::Error`.
3. **F5 — Manifest validation.** Reject empty `id`, self-dependency, duplicate deps at
   load time with a dedicated error variant.
4. **F6 — `Dag::topo_order()`.** Deterministic dependencies-before-dependents order
   (what §3 step 4 and cell_key's "in DAG order" need).
5. **F7 — Dependency bumps.** petgraph 0.6→0.8, thiserror 1→2, toml 0.8→1 if API-compatible;
   revert any that aren't.
6. **Docs.** ARCHITECTURE.md: domain-separation note in §2, quarantine-visibility +
   resource-envelope requirements in the T3 forward spec, golden-update-through-Phase-J
   policy in §4.2 area, metamorphic guidance in §1.2. decisions.md: D9 (domain-separated
   hashing, frozen contexts, re-key precedent), D10 (survey adoption map; T12/T13).
   Backlog: add T12 (frontier-scoped mutation testing), T13 (fuzz tier); fold quarantine
   visibility + resource envelopes into T3's entry.

## Out of scope
T3/T4 implementation; mutation/fuzz tiers themselves; any CLI.
