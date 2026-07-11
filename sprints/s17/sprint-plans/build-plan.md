# s17 Build Plan — Finalized - DO NOT EDIT
chained.rs (ChainState + append_ndjson_line); cache.rs (read_cache<T>). Open-once
MutationWriter/FuzzWriter replacing append_mutation/append_fuzz. Typed sidecar errors
(F4). Wire the 4 cache reads through read_cache (F6). 2-entry mutation chain-verify test.
D28. PR. Byte-preserving; generic ledger extraction deferred.
