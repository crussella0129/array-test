# Sprint s21 — Meta
- Sprint: 21
- Title: Perf cleanup + documentation currency (re-derived from code)
- Phase: loop
- Exit status: green
- Confidence: 1.0 (130 pass / 3 ignored; clippy -D warnings + fmt clean; behavior-preserving)

## Done
- Perf: audit_evidence folded two evidence-dir read_dir passes into one (build the stored
  stem set during the hash-check loop). Cold-path I/O, zero behavior change.
- Docs: README brought current — D1–D31, sprints s0–s20 (grouped), 130 tests, fuzz tier
  added to the status blurb. Other docs checked clean of stale markers.

## Judgment calls
- Left the audit_roots by_round clone alone (fixing it would ripple RootRecord::from_entries
  for negligible cold-path gain).
- Did NOT manufacture hot-path micro-opts: the real perf story is the frontier/caching
  economics, which is the core design. See D32.

## Provenance note
The external plan's perf/docs findings weren't committed; this sprint re-derives the work
from the code (measure-first), recorded as re-derived rather than pretending to the plan's
exact F-numbers.

## Next
s22-equivalent (deeper docs/archival) if wanted; a focused run_cell decomposition pass;
or T8b live Kani / T14 sprint-loops adapter (deferred side-quests).
