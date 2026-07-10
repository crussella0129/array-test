# s4 Test Plan — Finalized - DO NOT EDIT

AC1–AC28 stay green. New checks (unix; cells are `/bin/sh` scripts):

## T5 round orchestration
- [ ] **AC29** Fresh workspace, round 1: every cell executes (reused = 0), root written
  to `roots/R1.json`, all-Pass ⇒ green.
- [ ] **AC30** Round 2 with no changes: zero executions, all cells reused, **identical
  root** to round 1.
- [ ] **AC31** Change a leaf unit (no dependents): only that unit's cell executes;
  everything else is reused.
- [ ] **AC32** Change a root dependency: the full dependent closure re-executes
  (the "backwards" arrow emerging from closure-scope keys).
- [ ] **AC33** A failing cell ⇒ round not green; the Fail is cached and **reused** on the
  next unchanged round (deterministic failures don't re-run).
- [ ] **AC34** A nondeterministic cell is Quarantined, is NOT cached (re-executes next
  round), and the round is not green.
- [ ] **AC35** After a change, the new round's root covers exactly the planned cells —
  stale cell_keys from earlier rounds do not leak into the certificate.
- [ ] **AC36** Ledger after two rounds fully chain-verifies; reused entries carry
  `reused = true`.

## T11 CLI
- [ ] **AC37** `array-test run` exits 0 on a green round and 1 on a red one;
  `roots/R<k>.json` exists afterward.
- [ ] **AC38** `array-test verify` exits 0 on an untampered state dir and nonzero after
  a ledger byte is flipped.

## Exit condition
AC1–AC38 green, clippy clean → s4 exits green.
