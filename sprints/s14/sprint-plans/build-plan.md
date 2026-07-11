# s14 Build Plan — Finalized - DO NOT EDIT

1. Runner: recognized declared-env flag ARRAY_TEST_FS_READONLY=1 → private mount ns +
   mount_setattr(AT_RECURSIVE, RDONLY), fail-closed, probe-gated (D25).
2. Quickstart: contract-audit unit (T7b as command convention, closure scope).
3. Tests AC90–AC92; D25; records; PR.
