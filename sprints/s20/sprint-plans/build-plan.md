# s20 Build Plan

1. F18: `validate_unit_id(&str)` in manifest.rs; call from `Manifest::validate`. Reject
   empty, `/`/`\`, `..`, leading `.`, control/whitespace. Allow dotted namespacing.
2. F16: `safe_state_path(state_dir, ref) -> Result<PathBuf, JudgeError>` — reject absolute
   / non-Normal components; `UnsafePath { reference }` variant. Use at the repair-loop join.
3. F17: trust-boundary doc-comment on `run_repair`.
4. F21: Safety doc-comment on `unsafe fn make_root_readonly`.
5. Verify: cargo test, clippy -D warnings, fmt --check. Zero frozen surfaces.
