# s16 Research Report — Test Infrastructure

## Input
Refactoring plan §3.2. Highest-value item: F11 — three headline features (network
namespaces, read-only mounts, real Hypothesis property testing) gate on host
capability and `return` silently, so on ubuntu CI they report PASS without executing.
A regression in the runner's isolation setup would ship green.

## s16 scope (tight, high-value, low-risk)
- **F11**: convert the three silent skips to `#[ignore = "reason"]` so an incapable host
  reports *ignored*, never falsely *passed*; add a privileged CI job
  (`--privileged --cap-add=SYS_ADMIN`, pip install hypothesis) that runs
  `cargo test -- --ignored` and actually exercises them.
- **F12 (scoped)**: co-located parser edge-case tests where they pay the most and cost
  the least — `tap::parse_libtest` (separator-less lines, unknown verdicts, names
  containing the separator, empty output) and `hash::Hash::from_str` (round-trip +
  missing prefix / wrong length / non-hex).

## Deferred (documented, not dropped)
F9 (`tests/common/mod.rs` consolidation — a mechanical 11-file migration; real churn,
low urgency), F13 (Rust proptest), F14 (parse_args extraction) remain future test-infra
work. Doing F11 well now removes the one item that could let a real regression ship
green; the rest are coverage-depth, not correctness-masking.
