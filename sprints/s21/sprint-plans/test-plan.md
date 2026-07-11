# s21 Test Plan
- Perf: t16_audit (5 tests) is the behavior witness — the single-pass audit must produce
  identical problems/notes/counts, including the missing-evidence note and tamper detection.
- Docs: prose only; grep confirms no stale markers (D1–D8, s0–s10, 109 tests, VALID_SCOPES,
  BTreeMap<String, remain.
- Gate: 130 pass / 3 ignored; clippy -D warnings + fmt clean.
