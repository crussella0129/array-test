# Decisions Log

Cross-sprint architectural decision log (sprint-loops convention). Append-only.

## D1 — The regression array is a Merkle DAG of confirmations (s0)
**Context:** Schema shows a regression array growing exponentially across sprints/units.
**Decision:** Model each test as a *cell* keyed by a content hash of everything that can
affect it; record results as confirmations; commit the whole set as a Merkle root.
**Consequence:** Round cost scales with the *changed frontier*, not total history.
**Alternatives rejected:** Re-run-everything (doesn't scale); time-based caching (not
provable / not deterministic).

## D2 — Integration scope comes only from the declared DAG (s0)
**Context:** All-pairs/all-subsets integration is the combinatorial blow-up.
**Decision:** Integration cells exist only along declared `deps` edges, across a fixed
scope ladder UNIT→DIRECT→CLOSURE→E2E.
**Consequence:** Integration cells = O(edges), not O(2^units).

## D3 — Hermetic execution is mandatory (s0)
**Context:** Memoization is only valid if results are reproducible.
**Decision:** Pinned seeds, frozen clock, no ambient I/O, pinned toolchain hashed into the
cell key. A determinism meta-check quarantines non-reproducible cells.
**Consequence:** Stable keys → cache hits → the cost model in D1 holds.

## D4 — "Provable" = audit root (always) + property/contract tiers (scaled) (s0)
**Context:** User wants "provable." Full formal proof of all code is infeasible.
**Decision:** Always ship a hash-chained ledger whose green root certifies execution over
exact code. Layer property-based tests (∀-claims) and an optional model-checked formal tier
for critical units, recording the guarantee level per cell.
**Consequence:** Honest, deliverable provability with a clear upgrade path.

## D5 — One sprint = one regression round R_k (s0)
**Context:** Align the schema's R1…Rn with the sprint-loops protocol.
**Decision:** Each sprint's Test phase runs exactly one round; its green root is the gate
the next sprint reads (the schema's "loop back to current sprint").

## D6 — Adopt given/should/actual/expected + TAP as evidence (s1 research)
**Context:** Researched `crussella0129/riteway`, an AI-native testing framework built
around RITE (Readable, Isolated, Thorough, Explicit) and the "5 questions every unit test
must answer." Its assertion shape already forces tests to answer exactly what
`ARCHITECTURE.md §7` needs, and its output (TAP — Test Anything Protocol) is a
standardized, tool-compatible evidence format.
**Decision:** `tests/` are authored in the `given/should/actual/expected` shape; TAP
output is hashed into `evidence_hash` (§1.2, §2) instead of a bespoke format.
**Consequence:** No evidence format to invent or maintain; test authoring is
agent-legible by construction; the evidence contract is language-agnostic (TAP), so it
does not by itself dictate the implementation language — see **D8**, which supersedes
this decision's original lean toward Node/JS.

## D7 — Two-phase confirmation gate: deterministic AND judged, with a repair micro-loop (s1 research)
**Context:** Passing tests (Phase D) proves code doesn't crash and satisfies the
assertions someone wrote — it does not prove the code matches intent. `riteway ai`'s
judge-agent + N-run + threshold model checks that. User decision: these should not be two
independently-recorded tiers but a gate **in series** — `confirmed = det_status PASS AND
judge.rating >= threshold` — and a judge failure should trigger a scoped repair loop, not
a sprint-wide failure.
**Decision:** Add Phase J (judged) after Phase D (deterministic) in `ARCHITECTURE.md §4`.
Judge verdicts are recorded in their own hash-chained ledger (`judgments.ndjson`,
§7.3/§8) but are explicitly **excluded** from the Merkle root that backs the provability
claim (§7.1) — the root stays strictly reproducible; the judge layer is audited but not
"proved." A Phase-J failure spawns a Plan→Build→Test micro-loop scoped to the single unit
(§4.3), escalating to a sprint-level `failure-report.md` only if it exhausts a retry
budget.
**Consequence:** "Provable" stays honest (never let a statistical opinion masquerade as a
reproducible proof) while still gating on semantic/spec-faithfulness, not just
pass/fail. Fix cost for a rejected unit is bounded to that unit, not the whole sprint.
**Alternatives rejected:** Recording the judge tier as an independent, non-gating
annotation (weaker — a spec-unfaithful unit could still ship); folding the judge rating
into the Merkle root (would break reproducibility of the root itself).

## D8 — Rust core engine + Python (Hypothesis) property tier; resolves R-d (s1)
**Context:** D6's lean toward Node/JS was an accident of riteway being JS-native, not a
requirement of the architecture — TAP (D6) is language-agnostic. User is a primarily-Rust,
sometimes-Python developer; assessed Rust, Python, and Node/TS against the concrete
requirements in `ARCHITECTURE.md`: hermetic determinism (§6), `code_hash`/`cell_key`
hashing, DAG resolution (§1.3), property-based testing (§7.2), the optional formal tier
(§7.2), and toolchain-pinning stability (`toolchain_hash`, §2).
**Findings:** Rust wins on determinism-by-construction (no ambient globals), single
static-binary distribution (simplest possible `toolchain_hash`), raw throughput, and has
the only mature formal-verification story of the three (Kani model-checks real Rust,
directly serving the "provable" ambition in the original request). Python's `hypothesis`
is the strongest property-based testing tool available in any of the three ecosystems
(generation + shrinking).
**Decision:** The array-test engine — content addressing (T1), DAG resolver (T2),
hermetic runner (T3), ledger/Merkle root (T4), frontier selection (T5), judge gate (T9),
repair micro-loop (T10), CLI (T11) — is built in **Rust**. Property-based tests (T7) run
via **Python + Hypothesis**, invoked as a subprocess that emits TAP across the same
evidence boundary (D6) any other language's test runner would use. riteway is demoted
from "the toolchain" (D6's original lean) to one optional TAP-emitting adapter, usable if
and when a unit is written in JS — never a dependency of the core engine.
**Consequence:** Resolves R-d (s0 research report; s1 research report R-d). `code_hash`
inputs, hermeticity enforcement, and the CLI are all Rust from T1 onward; s1's build-plan
and test-plan (written before this decision) are amended in place with a visible note
rather than silently rewritten.
**Alternatives rejected:** Pure Rust (proptest/quickcheck are weaker than Hypothesis for
this specific job — small but real ergonomics cost); pure Python (weaker hermeticity
guarantees, slower at scale, much less mature formal-tier tooling than Kani).
