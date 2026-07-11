# Sprint s2 — Meta

- **Sprint:** 2
- **Title:** Testing-practice survey + design-taste refactor (hashing integrity, filesystem determinism)
- **Phase:** loop (research, build, test complete)
- **Started:** 2026-07-10
- **Exit status:** green
- **Confidence:** 1.0 (all AC1–AC18 green; one clippy finding fixed in-sprint)

## Goal
Survey established testing practice for anything the design is missing; review the s1
code with fresh eyes; refactor accordingly.

## Definition of done
- [x] Research report: 10-topic survey with adopt-now/later/rejected verdicts, plus
  code-review findings F1–F7 (`sprint-research/research-report.md`).
- [x] F1 — Domain-separated hashing: BLAKE3 `derive_key` contexts (`array-test/v1/...`,
  frozen) + RFC 6962-style 0x00/0x01 leaf/node role prefixes. The s1 hasher *claimed*
  domain separation in its docs while implementing none — the worst kind of defect in an
  integrity module.
- [x] F2/F3/F4 — Filesystem determinism: `/`-joined UTF-8 path normalization, string
  sort, loud rejection of non-UTF-8 names and symlinks; typed `CodeHashError`.
- [x] F5 — Manifest validation at load: empty id, self-dep, duplicate deps.
- [x] F6 — `Dag::topo_order()`: deterministic deps-before-dependents order (§3 step 4).
- [x] F7 — Dependency bumps: petgraph 0.8, thiserror 2, toml 1 — all compiled with zero
  code changes.
- [x] Docs: D9 (frozen contexts + re-key precedent), D10 (survey adoption map);
  ARCHITECTURE.md §1.2 metamorphic guidance, §2 domain-separation note, §4.2 golden
  policy, §6 quarantine visibility + resource envelopes; backlog T12
  (frontier-scoped mutation testing), T13 (fuzz tier).
- [x] Tests: 36/36 green (AC1–AC8 preserved through the re-key — they assert properties,
  not concrete hash values; AC9–AC18 new). clippy clean.
- [ ] Committed & pushed.

## Notable
All hash values changed this sprint (re-key). Safe exactly now — no ledger exists yet.
D9 records the precedent: hash-semantics changes ride ahead of the first ledger commit,
or they pay for a full re-confirmation of the array.

## Next sprint (s3) preview
T3 hermetic cell runner (with quarantine visibility + resource envelopes per D10) and
T4 confirmation ledger + Merkle root — the first sprint that can run a real `R_k`, at
which point the v1 hash contexts freeze for good.
