# s15 Research Report — Adopting the Refactoring Plan; Foundation & Hygiene

## Input
An external review (Super Z) produced a 42-finding plan (F1–F42) framed as sprints
s15–s22, all post-freeze extensions. Assessment: well-reasoned, findings real, framing
(sidecar/value per D20) correct. Adopted as the roadmap (D26), worked with per-finding
judgment rather than verbatim.

## s15 scope — the quick wins (highest impact-per-minute, zero frozen-surface risk)
F24 LICENSE, F7 repr(u8), F5 canonical_bytes, F15 subcommand msg, F10 env-race test,
F26 Cargo metadata, F27 gitignore/gitattributes, F8 curated lints.

## Two judgment calls worth recording
1. **F7's risk was overstated.** CellScope discriminants were already explicit, so the
   cast was stable and no truncation reachable. repr(u8) is still worth adding as
   *guaranteed* representation + reorder protection — done, described accurately.
2. **F8 full pedantic is 130 warnings, not ~30.** Dominated by noise
   (must_use_candidate / missing_errors_doc / module_name_repetitions). Under CI's
   `-D warnings` that is a disproportionate forced diff. Scoped to a curated high-signal
   set via `[lints.clippy]`. Full pedantic is a deliberate non-goal.

## Deferred to later plan sprints
s16 test infra (F9/F10.../F14), s17 HashChainedLedger extraction + O(N²) fix (F1/F4/F6/
F37), s18 type safety (F2), s19 decomposition (F3), s20 security (F16/F17/F18/F21/F23),
s21 perf (F36/F38-F42), s22 docs/archival (F28-F35).
