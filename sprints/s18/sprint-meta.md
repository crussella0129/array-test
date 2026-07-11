# Sprint s18 — Meta
- Sprint: 18
- Title: Enum-ify the manifest scope keys (F2)
- Phase: loop
- Exit status: green
- Confidence: 1.0 (130 pass / 3 ignored; clippy -D warnings + fmt clean; frozen bytes untouched, rot guard green)

## Done
- Manifest.tests: BTreeMap<String, TestSpec> -> BTreeMap<CellScope, TestSpec>.
- CellScope gains Serialize/Deserialize (rename_all = "lowercase"); `[tests.<scope>]`
  parses into the domain type; unknown scope is a parse error.
- Deleted VALID_SCOPES const + its runtime validation loop; round.rs drops the
  scope.as_str() round-trip (tests.get(&scope)).

## Freeze safety
Only `scope as u8` is hashed; discriminants unchanged. Rot guard (t15b_durable) green
confirms the durable ledger still verifies. See D30.

## Coverage
- New manifest-layer test: unknown scope fails to parse; the four valid scopes load.
- Existing t5b load-rejection test unchanged and still green.

## Next
s19 (F3 function decomposition), s21 (F36/F38–F42 perf), s22 (F28–F35 docs/archival).
