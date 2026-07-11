# s19 Test Plan
- No new behavior ⇒ no new acceptance tests; the sprint is proved by the existing suite
  staying green byte-for-byte.
- Key witnesses: t16_audit (audit decomposition), t12_mutation (mutation decomposition),
  t5b/round tests (resolve_cell), t15b_durable rot guard (nothing hashed moved).
- Gate: 130 pass / 3 ignored; clippy -D warnings + fmt clean.
