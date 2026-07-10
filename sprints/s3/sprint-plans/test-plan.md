# s3 Test Plan — Finalized - DO NOT EDIT

AC1–AC18 stay green. New checks (unix-targeted where subprocesses are involved):

## T3 runner
- [ ] **AC19** Exit 0 → `Pass`; nonzero exit → `Fail` (with code recorded).
- [ ] **AC20** Evidence captures stdout and stderr; same command twice → same
  `evidence_hash`; different output → different hash.
- [ ] **AC21** Parent environment does not leak: a var set in the test process is
  invisible to the cell; `ARRAY_TEST_SEED` and the hygiene set are present.
- [ ] **AC22** A cell exceeding its wall-clock envelope is killed and reported
  `TimedOut`, not `Fail`.
- [ ] **AC23** Determinism meta-check: a nondeterministic cell (reads `/dev/urandom`) →
  `Quarantined`; a deterministic cell → `Confirmed` with matching hashes.

## T4 ledger
- [ ] **AC24** Append N entries → `load` verifies the chain and returns them in order.
- [ ] **AC25** Tampering with any persisted byte of an entry → verification fails.
- [ ] **AC26** Array root is deterministic (same set → same root, regardless of append
  order) and sensitive (any status flip or added cell → different root).
- [ ] **AC27** Root record round-trips through `roots/R<k>.json`; `all_pass` is true iff
  every cell is `Pass` (Quarantined/TimedOut are NOT green).
- [ ] **AC28** Quarantined and TimedOut statuses are visible in loaded ledger entries
  (D10 visibility requirement).

## Exit condition
AC1–AC28 green, clippy clean → s3 exits green.
