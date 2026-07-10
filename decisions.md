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

## D9 — Domain-separated hashing with frozen v1 contexts; re-key precedent (s2)
**Context:** s2 code review (research report F1–F4) found the s1 hasher claimed domain
separation in its docs but implemented none, plus three filesystem-determinism holes
(lossy non-UTF-8 path conversion, platform-dependent separators/sort order, silently
followed symlinks). The provability claim (§7.1) rests on these keys being unambiguous
by construction.
**Decision:** Every hash derives under a named BLAKE3 `derive_key` context
(`array-test/v1/...`), with RFC 6962-style role prefixes (`0x00` leaf / `0x01` node) so
leaves and interior nodes can never collide even under a shared context. Paths are
normalized to `/`-joined UTF-8, sorted as strings; non-UTF-8 names and symlinks are
rejected loudly. Contexts are **frozen**: once a ledger root commits to v1 hashes,
changing any context or structural rule is a formal re-key event (new version namespace,
full re-confirmation).
**Consequence:** All hash values changed. Safe exactly now — no ledger exists yet (T4
unbuilt), so nothing committed refers to the old values. Precedent recorded: hash
semantics changes ride ahead of the first ledger commit, or they pay for a full re-key.
**Alternatives rejected:** Role-disjoint context table alone (a convention, not a
construction — one future refactor away from silently breaking); post-hoc migration
tooling (buys nothing while the ledger is unbuilt).

## D10 — Testing-practice survey adoption map (s2)
**Context:** Survey of established testing practice (s2 research report §2), each item
mapped against the architecture.
**Decision:**
- *Adopted now:* Merkle domain separation (→ D9); cross-platform filesystem determinism
  (→ D9).
- *Adopted later:* frontier-scoped **mutation testing** (backlog T12 — content-addressing
  makes mutation incremental: only dirty units re-mutate, scores memoized by `code_hash`);
  **fuzz tier** (backlog T13 — corpus as content-addressed fixtures); **quarantine as
  visible ledger state** and **per-scope resource envelopes** (folded into T3's spec);
  coverage as evidence *metadata* (T3 flag, explicitly never a gate — Goodhart).
- *Documented conventions:* metamorphic relations in `contract.toml [properties]` when no
  oracle exists (also an input to Phase-J judge prompts, R-e); golden/snapshot updates
  route through Phase J as semantic events, never auto-accepted.
- *Validated as already core:* frontier selection (= industry content-addressed action
  caching à la Bazel/Buck2/Nix), scope ladder (= Google test-size taxonomy), determinism
  meta-check (= flaky-test literature's causes, pinned or banned).
**Consequence:** The design gained two backlog tiers and several spec clauses without any
architectural change — the survey confirmed the array's shape and sharpened its edges.

## D11 — Library-first embedding contract; sprint-loops is a consumer, not a dependency (s3)
**Context:** array-test is intended to power the Test phase of the sprint-loops protocol,
but must remain standalone — usable by anyone, embeddable in any application.
**Decision:** One-directional coupling. array-test is a Rust library (plus a thin CLI,
T11) that never references sprint-loops — no paths, no phase names, no knowledge a
consumer exists. Consumers integrate against a **stable output contract**: (1) green iff
the array root is all-PASS; (2) `roots/R<k>.json` round certificates; (3) the
independently re-verifiable hash-chained `confirmations.ndjson`; (4) hash-committed TAP
evidence. The sprint-loops Test-phase shim (backlog T14) consumes these and produces
sprint-loops' own artifacts (`test-report.md`/`failure-report.md`) on its side of the
boundary.
**Consequence:** Anyone holding the ledger file can verify the chain and root with zero
trust in the runner; sprint-loops conventions can change without touching array-test, and
vice versa.
**Alternatives rejected:** Building sprint-loops' file conventions into array-test
(couples every embedder to one consumer's layout); a shared "integration crate" (a second
thing to version for what a stable file format already does).

## D12 — v1 runner hermeticity level: env hygiene + meta-check, not a sandbox (s3)
**Context:** ARCHITECTURE.md §6 demands "no ambient I/O", but full isolation (network
namespaces/seccomp, memory rlimits) is real engineering that shouldn't block the first
runnable round.
**Decision:** T3 v1 enforces: cleared environment (declared vars + hygiene set
`TZ/LC_ALL/SOURCE_DATE_EPOCH` + `ARRAY_TEST_SEED`), no stdin, wall-clock envelope with
**process-group** kill (killing only the direct child leaves grandchildren running and
holding pipes), and the run-twice determinism meta-check with ledger-visible quarantine.
Network and memory isolation are deferred to T3b and recorded as gap **R-g**: until T3b,
a cell's determinism claim is "meta-checked", not "sandbox-guaranteed".
**Consequence:** Honest labeling of the guarantee level; the meta-check catches the
nondeterminism that ambient I/O actually causes, which is the failure mode that matters
for cache validity.

## D13 — Round semantics: closure-scope cells, cache policy, per-round roots (s4)
**Context:** Wiring T5 required locking four semantics (s4 research report §2).
**Decision:**
1. **v1 cells are CLOSURE-scope** — one per unit with `[test]`; the key includes the
   transitive dep closure's `code_hash`es in topo order. The "backwards" arrow is
   thereby *emergent*: a dependency change re-keys every transitive dependent, putting
   exactly the impact set into the frontier with no separate impact machinery.
2. **Cache: Pass AND Fail are reusable forever per key** (a deterministic failure is a
   fact, not something to re-check); **Quarantined and TimedOut never enter the cache**
   (irreproducible / host-dependent).
3. **The round root commits to the round's planned cells only.** A whole-ledger
   "latest per key" root would leak stale keys forever after any change. History keeps
   everything; the certificate speaks only for now.
4. **Reused confirmations are ledger entries too**, flagged `reused` inside the chained
   hash — every round is self-contained in history and the inheritance is auditable.
**Gaps recorded:** R-h — toolchain hash defaults to an explicit "unpinned" sentinel
until a real pinning story exists; T15 — true self-hosting blocked on TAP-clean output
(cargo prints timings, which the meta-check would correctly quarantine).

## D14 — Evidence determinism is produced at the source, never by normalization (s5)
**Context:** Self-hosting requires cells wrapping libtest/cargo output, which contains
wall-clock timings — the meta-check correctly quarantines raw wrapping.
**Decision:** The fix is the `array-test tap` adapter: the *cell's command* emits
minimal, sorted, timing-free TAP; the runner keeps hashing exactly what the cell
emitted, byte for byte. The adapter is part of the test definition (inside
`test_def_hash`), not the trust boundary. Normalizing/stripping evidence at the hasher
is rejected on principle: it makes the hash stop committing to what the cell emitted,
and every normalization rule is a new place for a flake to hide. Corollary encoded in
the adapter: a nonzero inner exit with no parsed failure synthesizes a `not ok` —
silence never reads as success.
**Consequence:** Self-hosting works (T15 landed: array-test certifies its own T2 suite
green, reuses it on the next round) and the meta-check keeps full power — if the
adapter itself ever emits instability, the cell quarantines, which is correct.
**Note:** the self-host cell runs the prebuilt libtest binary directly rather than
`cargo test` — cargo holds the build-dir lock for its whole session (inner cargo would
deadlock the outer), and the direct binary needs no PATH/HOME at all: strictly more
hermetic. Contexts remain formally frozen-on-first-durable-ledger (s5 research §5).

## D15 — Scope ladder semantics: keys ARE scopes; gating is visible state (s6)
**Context:** T5b generalizes D13.1's closure-only cells to the full ladder (§1.4).
**Decision:**
1. The scope decides which `code_hash`es enter the cell key — that IS its meaning:
   `unit` none, `direct` direct deps, `closure` transitive closure, `e2e` every unit in
   the workspace ("end-to-end depends on everything" taken literally). The scope itself
   is hashed in (SCOPE-domain leaf), so one test at two scopes cannot collide.
2. Declaration: `[tests.unit|direct|closure|e2e]`; legacy `[test]` = `[tests.closure]`
   (both together rejected). `[tests.e2e]` doubles as the entrypoint declaration.
3. Fail-fast tiers: unit → direct → closure → e2e. Once a completed tier holds a
   non-Pass, higher-tier cells are recorded **`Skipped`** — a ledger-visible status,
   never cached, not green (same doctrine as quarantine: skipping is state, not
   silence). Within-tier siblings still run; a reused Fail gates like a fresh one.
4. Per-scope wall-clock defaults (10/30/60/300s); memory caps opt-in per test.
**Design note surfaced by testing:** `code_hash` covers src+contract, not the manifest,
so byte-identical units share cell keys and dedup through the cache. Correct — identical
content is identical work — but fixture authors (and future doc readers) should know.

## D16 — Sandbox levels, probed and recorded; toolchain.lock closes R-h (s6)
**Context:** T3b (D12's deferred gap) and R-h (unpinned toolchain sentinel).
**Decision:**
- **Memory:** `mem_limit_mb` → `RLIMIT_AS` in pre_exec; breach = allocation failure
  inside the cell = `Fail` (a red cell, not a timeout).
- **Network:** one-time probe for netns capability (`CLONE_NEWNET`, else
  `CLONE_NEWUSER|CLONE_NEWNET`); on success **every** cell runs in a fresh namespace
  (loopback only) and pre_exec fails closed — a cell that cannot be isolated does not
  run. On failure, env-hygiene level as before. Either way the achieved level is
  recorded **per confirmation** in the chained ledger (`isolation: env_only |
  net_isolated`) — D12's "the ledger records which guarantee level applied", now real.
- **Toolchain:** explicit `--toolchain-hash` > `<units-dir>/toolchain.lock` bytes
  (TOOLCHAIN-domain leaf) > unpinned sentinel. Mechanism, not policy: what identifies a
  toolchain is the consumer's business to write into the file. R-h closed at the
  mechanism level.
**Remaining gap:** filesystem read scoping — the last R-g fragment; the meta-check
polices what the sandbox doesn't block.

## D17 — Phase J: the judge is a command; judgments get confirmation economics (s7)
**Context:** T9 lands D7's judge gate. An LLM judge needs network and is not
bit-deterministic — exactly what the det root excludes.
**Decision:**
- The judge is a **command** (`judge.toml`: command/runs/threshold/min_rating),
  receiving `ARRAY_TEST_UNIT_DIR/_UNIT_ID/_SCOPE/_EVIDENCE/_CONTRACT`, writing a
  critique to stdout whose last line is `rating: <0-100>`. Scripted judges make the
  protocol testable without an LLM; an LLM judge is just a different command.
- Judge identity is pinned: `judge_hash` over command+config (R-f) — a changed prompt
  is a new judge.
- **Judgments get the same content-addressed economics as confirmations:** verdicts
  cached by `(cell_key, judge_hash)` (unchanged cell + unchanged judge = no re-judging),
  recorded in their own hash-chained `judgments.ndjson` with critique transcripts under
  `ledger/critiques/` — audited, never rooted (§7.3 upheld).
- Phase J runs only over a det-green round; only det-Pass cells are judged (§4.2).
- **Guarantee levels** (`example|property|proved`) are declarations: hashed into
  `test_def_hash` (a changed claim re-keys), recorded per confirmation, audited by
  Phase J — never "verified" by the engine, which cannot.
- **Evidence store** (audit gap found during design): executed cells persist their
  exact framed evidence bytes content-addressed under `state/evidence/` — a root is now
  backed by retrievable, re-hashable evidence, not hashes of discarded data.
**T8 note:** `proved` ships as schema; running Kani is environment-gated → T8b.

## D18 — The repair micro-loop is just more rounds (s7)
**Context:** T10 lands §4.3.
**Decision:** On judge rejection, the repair command (`[repair]` in judge.toml,
receiving `ARRAY_TEST_UNIT_DIR` + `ARRAY_TEST_CRITIQUE`) edits the unit; the next
attempt is simply **another det round** — the changed unit re-keys, the frontier re-runs
exactly what moved, Phase J re-judges only moved keys (cache handles the rest). Attempts
are ordinary numbered rounds in history; no special loop state exists anywhere. Budget
exhausted (or no repair configured) → consumer-agnostic failure record
`ledger/failures/R<k>-judgment.md` with critique refs (T14's shim can translate it to
sprint-loops' `failure-report.md`).
**Consequence:** §4.3's "the micro-loop is a local fixed-point search, not a separate
code path" turned out to be literally implementable: the loop body is `run_round`.
