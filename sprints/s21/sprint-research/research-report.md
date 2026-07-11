# s21 Research — perf + docs, re-derived

The plan's final clusters were performance and docs/archival. The plan file is not in the
repo, so I scanned the code directly rather than reconstruct F-numbers from memory.

## Perf
Grepped for the usual structural costs (clones, collects, repeated read_dir/read_to_string)
and read the longest / most I/O-heavy paths. Findings:
- audit_evidence reads the evidence dir twice — a genuine redundant enumeration, trivially
  merged into one pass with identical semantics. Worth doing.
- audit_roots clones every LedgerEntry into a by_round map. Real allocation, but removing it
  means RootRecord::from_entries / array_root must accept &[&LedgerEntry], a wider API change
  for a cold path. Not worth the ripple; left alone.
- The hot paths (run_round→resolve_cell per cell; run_mutation copy-per-mutant) have no
  redundant work to remove — a mutant genuinely needs an isolated workspace copy, a cell
  genuinely needs its cache lookup. The frontier economics (skip unchanged cells) are the
  performance design, already in place (D13/s17 O(1) appends). Manufacturing micro-opts here
  would add risk without measurable benefit.

## Docs
The README had drifted badly: decision log cited as "D1–D8" (now D31), sprint state stopped
at s10, test count "109" (now 130), and the fuzz tier (s13) was never mentioned in the
status blurb. For a repo offered as a GitHub template (D22), the front page describing an
earlier version of itself is a real defect. ARCHITECTURE/SCHEMA/TEMPLATE checked clean of
the stale markers.

## Doctrine
Measure before optimizing; document divergence from the plan (re-derived, not invented);
never manufacture work to fill a sprint slot — one honest structural fix + real doc currency
beats a page of speculative micro-tuning.
