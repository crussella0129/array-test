# s2 Test Plan — Finalized - DO NOT EDIT

All s1 acceptance checks (AC1–AC8) must stay green through the refactor — they assert
properties, not concrete hash values, so they survive the re-key (research report §4).
New checks:

## Domain separation (F1)
- [ ] **AC9** Same bytes hashed under two different contexts produce different hashes.
- [ ] **AC10** A `node` over parts and a `leaf` over the identical concatenated bytes
  differ (leaf/node confusion is impossible).
- [ ] **AC11** Empty inputs are domain-distinct: `node(ctx_a, [])` ≠ `node(ctx_b, [])`.

## Filesystem determinism (F2/F3/F4)
- [ ] **AC12** A non-UTF-8 filename inside `src/` is rejected with an error (unix-only test).
- [ ] **AC13** A symlink inside `src/` is rejected with an error (unix-only test).
- [ ] **AC14** Nested-directory paths hash identically across separator conventions
  (normalization test: hash equals itself when built via different path spellings) and a
  file moved between directories changes `code_hash`.

## Manifest validation (F5)
- [ ] **AC15** Self-dependency rejected at load.
- [ ] **AC16** Duplicate deps rejected at load.
- [ ] **AC17** Empty `id` rejected at load.

## DAG order (F6)
- [ ] **AC18** `topo_order()` places every dependency before every dependent on the
  fixture graph, and is byte-identical across repeated builds of the same units.

## Exit condition
AC1–AC18 green, `cargo clippy --all-targets` clean → s2 exits green.
