# Sprint s13 — Meta

- **Sprint:** 13
- **Title:** T13 — the fuzz tier; fixtures become real
- **Phase:** loop
- **Exit status:** green
- **Confidence:** 1.0 (118/118; one design defect caught by the sprint's own cache
  test — see Notable)

## Definition of done
- [x] `compute_fixtures_hash`: `<unit>/fixtures/` fills the frozen key slot; no
  fixture files ⇒ sentinel (pre-T13 keys preserved). Per-unit wiring in plan_round.
- [x] `src/fuzz.rs`: fuzzer-as-command (exit 65 = findings into `fixtures/fuzz/`),
  chained `fuzz.ndjson`, clean-result cache `(code_hash, fuzzer_hash, fixtures_hash)`,
  audit coverage; `fuzz` CLI verb.
- [x] AC85–AC89 green (5 new tests; 118 total), fmt+clippy(-D warnings) clean.
- [ ] Committed to dev; PR to main.

## Notable
AC87's cache test caught a real design defect on first run: creating the (empty)
corpus directory changed `fixtures_hash`, silently invalidating the cache on every
second run. Fix with the right semantics, not a workaround: the hash covers fixture
*content* — an empty tree and an absent one are the same claim. The suite keeps
auditing the design, not just the code.

Also the sprint's satisfying moment: the fuzz loop needed no engine coupling at all —
findings land in the corpus, content addressing does the rest, exactly as the s2
survey predicted ("corpus growth re-keys cells naturally").
