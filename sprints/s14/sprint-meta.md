# Sprint s14 — Meta

- **Sprint:** 14
- **Title:** T3c read-only cells + T7b contract enforcement closed
- **Phase:** loop
- **Exit status:** green
- **Confidence:** 1.0 (121/121; one clippy nit fixed; RO behavior verified live)

## Definition of done
- [x] The freeze's first real constraint met head-on: no relayout of `test_def` —
  declared env (already hashed) becomes the per-test extension channel (D25).
- [x] `ARRAY_TEST_FS_READONLY=1`: fresh private mount ns + `mount_setattr(AT_RECURSIVE,
  RDONLY)`, fail-closed, probe-gated; proven live (writes fail everywhere incl. /tmp,
  reads work, no host leak).
- [x] T7b closed per D20: `contract-audit` unit in the quickstart enforces dependency
  contract post-invariants as a closure-scope command, CI-guarded.
- [x] AC90–AC92 green (121 total), fmt + clippy(-D warnings) clean.
- [ ] Committed to dev; PR to main.

## Notable
The sprint began with the freeze refusing a design: appending a byte to test_def's
canonical layout is exactly what D21 forbids. The refusal produced a better mechanism
than the plan had — declared env was already the extensible, hashed, per-test channel.
Constraints that bite are constraints that work.

## Remaining backlog after s14
- **T8b (live Kani):** environment-gated — the toolchain is a multi-GB install beyond
  this sandbox's sensible budget; schema + recipe stand ready (s7).
- **R-g fragment:** read scoping to declared inputs (low priority; meta-checked).
- **T14:** parked on the user's sprint-loops-side-or-here decision.
