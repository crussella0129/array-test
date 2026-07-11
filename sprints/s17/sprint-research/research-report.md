# s17 Research Report — The Sidecar Dedup (F1) + F4/F6

## Re-audit correction
F1 claimed the O(N^2) re-read-per-append bug in 3 of 4 hash-chained ledgers. At s17 it's
2 of 4: judge.rs was already fixed in s9 (JudgmentWriter is open-once). Live bug:
mutation.rs + fuzz.rs.

## Decision (D28)
Fix the bug the proven way (open-once writers holding ChainState); share the bookkeeping
primitive (chained::ChainState + append_ndjson_line), NOT a generic-over-entry-type
ledger. Rationale: the 4 entry layouts differ, one is freeze-locked, and a shared trait
would be more machinery than a 4-instance pattern earns. Byte-preserving.

F6: cache::read_cache<T> for the 4 cache sites; fixes fuzz unwrap_or_default (hid I/O
errors as ""). NotFound = silent miss; corruption surfaced on stderr.
F4: typed Spawn/Malformed/ChainBroken variants in MutationError/FuzzError.

## Deferred (documented, not dropped)
The fully-generic HashChainedLedger<T> extraction (the maintainability-only half of F1).

## Coverage
Sidecar layouts aren't under the rot guard; added a 2-entry chain-verify assertion to
the two-unit mutation test as the multi-append witness.
