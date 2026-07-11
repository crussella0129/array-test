# s18 Research — F2, stringly-typed manifest scopes

The manifest carried the scope set twice: as string keys in `tests: BTreeMap<String,
TestSpec>` (validated against `VALID_SCOPES` at load) and as the `CellScope` enum (the
domain type actually hashed). `round.rs` bridged them with `tests.get(scope.as_str())`.

"Make illegal states unrepresentable" (Minsky/King) says the closed set should be one
typed thing. `CellScope` already exists, is `#[repr(u8)]` with frozen discriminants, and
already derives `Ord`/`Hash` (so it is a valid `BTreeMap` key). Deriving serde on it — with
`rename_all = "lowercase"` to match the TOML surface — lets `[tests.<scope>]` deserialize
straight into the enum, so an unknown scope is a *parse* error and the hand-rolled check
disappears.

The one risk worth checking twice: `hash.rs` is a frozen module. But the freeze is over
*hashed bytes*, and only `scope as u8` is ever hashed. Adding a derive and changing a map
key type in the manifest does not move a single byte into any `cell_key`. The durable
rot-guard test is the empirical proof and must stay green.

Considered a struct-of-`Option<TestSpec>` (`unit`/`direct`/`closure`/`e2e`) with
`deny_unknown_fields`. It is arguably more explicit but changes every `.get`/`.values`
call site and enlarges the diff for no byte-level benefit; the enum-keyed map is the
smaller, obviously-neutral change. `toml` handles enum map keys, so there is no wire risk.
