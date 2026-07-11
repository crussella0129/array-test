# s18 Build Plan
1. hash.rs: derive Serialize/Deserialize on CellScope with `rename_all = "lowercase"`;
   import serde. Doc the freeze-neutrality (only `scope as u8` is hashed).
2. manifest.rs: `tests: BTreeMap<CellScope, TestSpec>`; import CellScope; delete
   VALID_SCOPES + the unknown-scope validation loop; `contains_key(&CellScope::Closure)`.
3. round.rs: `tests.get(&scope)`.
4. Test: manifest-layer unknown-scope-fails-to-parse + four-valid-scopes-load.
5. Verify: build, clippy -D warnings, fmt --check, full suite incl. rot guard.
