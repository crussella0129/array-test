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

## D19 — Verify everything: the full audit is a library function (s8)
**Context:** `verify` had fallen behind the state it guards — it checked the
confirmations chain and the latest root while judgments, older roots, and the evidence
store went unaudited. Separately, a dev-loop shell "failure" (a `grep -c` pipeline
exiting 1 on a count of 0 — i.e., failing precisely because everything passed) supplied
the doctrine: **success must never read as failure**, the dual of D14's "silence never
reads as success". Exit semantics are consumer contract surface.
**Decision:** `audit::full_audit(state_dir)` — library-first (D11: embedders get the
trust tool; the CLI's `verify` is one caller) — checks the confirmations chain, EVERY
root certificate (root, cells, all_pass recomputed from that round's entries),
the judgments chain, and the evidence store (every file re-hashed against its content
address). Integrity violations (`problems`, nonzero exit) are strictly separated from
informational `notes` (e.g. quarantined/skipped evidence legitimately absent from the
store) — the two never mix, in either direction.
**Also:** `examples/quickstart/` committed as the D11 adoption surface — a real
two-unit workspace with a walkthrough README and a `judge.toml.example`, guarded by an
integration test so the example cannot rot silently.
**Test-fixture lesson recorded:** a tamper test tried to "forge" R1's `all_pass` from
false to true and found it already true — correctly, because the judge (not det) had
rejected that round and certificates are Phase-D-only by design (D7). The forgery that
matters is the root hash itself; the test now flips that.

## D20 — Sequencing determination: T15b next; extension is by sidecar and by value (s9)
**Context:** User asked whether T7b/T8b/T12/T13/T3c must precede T15b (durable ledger →
v1 context freeze) "so 1.0 proves itself fully," or whether they're separable.
**Determination: separable — T15b is next.** The freeze locks context strings and byte
*layouts*, and explicitly permits adding contexts. Walking each deferred tier: T7b's
contract enforcement is a command (already inside `test_def`); T8b's `proof_hash` and
T12's mutation scores follow the judgments-ledger precedent — new evidence classes get
their own hash-chained **sidecar** keyed by `cell_key` (additive, post-freeze-legal);
T13's corpus is fixtures (`fixtures_hash` already in the key); T3c is a new
`IsolationLevel` *value*, not a new layout. Doctrine: **post-freeze extension happens by
sidecar and by value, never by relayout.** The judgments ledger wasn't just a feature —
it was the extension mechanism. The deferred tiers prove 1.0's *claims*; T15b makes
1.0's *promise* (stable keys) — and shipping the promise first makes the tiers' later
results durable.
**Corollaries enforced this sprint (last free re-key):** F8 sentinel hygiene (skipped
cells get a `no-evidence` domain; unpinned toolchain properly TOOLCHAIN-domained).
**Frozen constants recorded:** per-scope timeout defaults (10/30/60/300s) are hashed
into `test_def` via the effective timeout, so they freeze with v1; changing them
post-freeze is a re-key event.
**Review findings applied (F8–F16):** quarantine now stores BOTH disagreeing
transcripts (the one status whose meaning is "these disagreed" no longer discards its
evidence); round numbers derive from the ledger, not the roots dir (crash between
append and certificate-write can no longer merge two attempts under one number);
`Ledger::record(ConfirmationInput)` replaces the 8-arg method; judgments appends are
open-once O(1); `manifest.sprint` optional (D11 polish); audit notes certificate-less
rounds; ARCHITECTURE §7.4 records the trust model — integrity you verify, truthfulness
you reproduce (re-run from an empty cache and byte-compare roots).

## D21 — THE v1 FREEZE: the durable ledger exists; version 1.0.0 (s10)
**Context:** T15b. D9 defined the trigger: the first durable committed ledger freezes
the `array-test/v1/*` contexts and every structural byte layout.
**The artifact:** `selfhost/state` — array-test testing itself through its own front
door. Three units (`selfhost.tap`, `selfhost.run`, `selfhost.verify`) drive the built
CLI via a **relative PATH entry** (`../../../target/debug`), which is what makes a
committed workspace machine-independent: exec resolves relative PATH against the
cell's cwd, and cell keys contain no paths at all. Even the *inner* round's root that
`selfhost.run` asserts is machine-independent (fixed fixture content, seed 0, sentinel
toolchain). Two founding rounds are committed: R1 (3 executed) and R2 (3 reused,
byte-identical root `blake3:70258f45…`), plus cache and evidence store; a rot-guard
test full-audits the committed state on every CI run, so the past cannot be edited
without failing the suite.
**The freeze:** as of this commit, every `array-test/v1/*` context, the 0x00/0x01 role
prefixes, the evidence framing, and the ledger/judgment/root canonical byte layouts
are **frozen**. Extension is by sidecar and by value (D20); relayout requires a v2
namespace and a full re-key. Frozen constants include the per-scope timeout defaults
(10/30/60/300s).
**Version: 1.0.0.** 1.0 here *means* the promise: keys stable forever. The deferred
guarantee tiers (T7b/T8b/T12/T13/T3c) extend against these stable keys per D20.
**Honest caveats carried:** `toolchain.lock` pins the producing environment's rustc —
other environments regenerate it (new keys, appended rounds, history unbroken);
selfhost scripts assume a POSIX Linux userland.

## D22 — The repo is a template, in two layers; instances share the hash language (s11)
**Context:** User observation after the freeze: a self-verified kernel makes the repo
itself a template (e.g. a GitHub template repository).
**Determination: correct, with one word sharpened.** The kernel is *self-certifying*
(integrity verified, truthfulness reproducible via §7.4's empty-cache re-run protocol,
self-hosting green), not *proven correct* — the proved tier (T8b) remains future work.
With that precision, the repo is two templates layered:
- **Layer A — the verification kernel:** frozen engine + founding ledger + rot guard.
  An instance writes units and has provable regression from commit one. Because v1
  contexts are frozen, **all instances speak the same hash language**: any v1 ledger is
  verifiable by any v1 binary, anywhere.
- **Layer B — the method scaffold:** the sprint-loops working state (decisions,
  sprints/, agent-tasks/) — not documentation about the work but the working memory
  that produced it.
**Mechanics:** `docs/TEMPLATE.md` records the layers, the instantiation steps, and the
**genesis ritual** — an instance resets `selfhost/state`, re-pins its toolchain, runs
two founding rounds, and commits its own D21 moment; the rot guard then protects *its*
history. CI (`.github/workflows/ci.yml`) keeps the template's promise live: every push
re-audits the committed ledger (fmt + build + test + clippy -D warnings). Flipping the
GitHub "Template repository" setting is a human step, noted in the doc.

## D23 — The mutation tier: who tests the tests, frontier-scoped (s12, T12)
**Context:** A green array certifies "all tests pass" — only as strong as the tests
(s2 §2.5). First post-freeze extension; must prove D20's doctrine.
**Decision:**
- **The mutator is a command** (`mutation.toml`: command/mutants/min_score — the
  judge/repair pattern a third time). It receives a scratch COPY of a unit and an
  index; language awareness (cargo-mutants, sed, an LLM) lives in the mutator, never
  the engine. `mutator_hash` pins its identity (R-f logic). Exit 64 = "no mutant at
  this index"; a mutant that doesn't change `code_hash` isn't a mutant (skipped).
- **Killed ⇔ a full round over the mutant workspace goes red** — deliberately broader
  than "the unit's own cells failed": a dependent's closure-scope cell catching the
  mutant is the integration lattice doing its job, and it counts. Baseline must be
  green first. Scratch rounds share one cache (content-addressed ⇒ sharing is safe by
  construction), so the frontier economics apply inside the mutation run too.
- **Memoization key = (code_hash, mutator_hash, baseline_root).** The baseline root IS
  a commitment to the whole detection surface, so scores re-compute exactly when any
  test, dep, seed, or toolchain changes — and never otherwise. Only the dirty frontier
  re-mutates: T12's founding promise (s2 §2.5), kept.
- **Sidecar everything (D20 proven):** new contexts (`mutator`, `mutation-entry`,
  `mutation-genesis`), hash-chained `mutations.ndjson`, audit coverage, cache under
  `mutation-cache/` — zero frozen surfaces touched.
- **CLI:** a separate `mutate` verb (deliberately expensive; never ambushes `run`);
  exit 0 iff every mutated unit is mutation-strong.
**Proven by test:** a content-checking unit kills 2/2 mutants (strong); a vacuous
`true` test lets 2/2 survive (weak — the exact pathology the tier exists to expose);
an unchanged workspace re-mutates nothing; tampering with a recorded score breaks the
chain.

## D24 — The fuzz tier; fixtures become real (s13, T13)
**Context:** `fixtures_hash` had been a frozen key slot filled with a sentinel since
s4. A fuzz corpus IS a fixture set (s2 §2.6).
**Decision:**
- **Fixtures:** `<unit>/fixtures/` content-hashes into the slot (same normalized,
  symlink-rejecting walk as `code_hash`; root under the existing FIXTURES context —
  role prefixes keep it distinct from the sentinel). **No fixture files ⇒ the
  sentinel** — the hash covers content, so an empty/absent dir are the same claim and
  every pre-T13 workspace keeps its keys. Value-level change to a frozen slot;
  D20-legal. (First implementation hashed empty-present ≠ absent; the cache test
  caught it — creating a corpus dir must be hash-neutral.)
- **The fuzzer is a command** (fourth use of the pattern): `fuzz.toml`
  command/budget_secs; exit 0 clean, exit 65 = findings **written into
  `fixtures/fuzz/`**. The loop closes through content addressing exactly as s2
  predicted: findings move `fixtures_hash` → cells re-key → the next round tests the
  grown corpus. No coupling beyond the filesystem.
- Clean results cache under `(code_hash, fuzzer_hash, fixtures_hash)`; the contract
  requires seed-determinism within budget (a nondeterministic fuzzer only wastes its
  own cache). Sidecar `fuzz.ndjson` + contexts + audit coverage; `fuzz` CLI verb.
**Proven by test:** a finding grows the corpus and flips the next round red until the
bug is fixed; clean units cache; tampered entries break the chain.

## D25 — Declared env is the per-test extension channel; opt-in read-only cells (s14)
**Context:** T3c wanted a per-test flag, but appending a field to `test_def`'s
canonical bytes is a relayout of a frozen surface — forbidden by D21. The freeze
showed its teeth, and the answer was already in the layout: **declared env is hashed
into `test_def`**, making it the open per-test key-value extension channel that was
there all along. Engine-recognized flags live under `ARRAY_TEST_*` names.
**T3c:** a cell declaring `ARRAY_TEST_FS_READONLY = "1"` runs in a fresh private mount
namespace with `mount_setattr(AT_RECURSIVE, RDONLY)` flipping every mount read-only —
fail-closed per D16 (declared-but-unsupported ⇒ the cell does not run), probe-gated
(`fs_readonly_supported()`), propagation made private so nothing leaks to the host.
Proven live: writes fail everywhere including /tmp, reads work, host unaffected. The
committed quickstart documents but does not declare the flag (unprivileged CI runners
lack mount namespaces; portability of the example wins). Remaining R-g fragment,
recorded honestly: *read scoping to declared inputs only* stays open — the meta-check
polices what reads smuggle in.
**T7b closed as D20 determined:** enforcement is a command. The quickstart gains
`contract-audit`, a closure-scope cell that checks its dependencies' contract
post-invariants and re-keys whenever any of them change (contracts are inside
`code_hash`). The contract tier is a convention with a live, CI-guarded example — not
an engine feature, which is exactly the point.

## D26 — Adopt the Super Z refactoring plan as the s15–s22 roadmap (s15)
**Context:** An external code-review pass (Super Z) produced a 42-finding refactoring
plan framed as sprints s15–s22, all post-freeze extensions. The plan is well-reasoned
and its findings are real; adopting it as the roadmap.
**Decision:** Work the plan in its proposed order, but apply engineering judgment per
finding rather than executing verbatim — the plan invites this (§1: "a clippy pedantic
run will surface additional items"). s15 = foundation & hygiene quick wins:
- **F24 LICENSE (MIT)** — the highest-impact item: D22's template ambition is legally
  void without it. Copyright crussella0129.
- **F7 `#[repr(u8)]` on `CellScope`** — freeze-hardening. *Correcting the plan's
  framing:* the discriminants were already explicit (`Unit = 0…`), so the `as u8` cast
  was already stable and no truncation was reachable; `#[repr(u8)]` pins the
  representation as *guaranteed* and forbids a future reorder from silently moving a
  cell key. Byte-preserving; rot guard confirms.
- **F5** `canonical_bytes` now takes `&ConfirmationInput` (10 args → 3, suppression
  gone), byte layout unchanged.
- **F15** subcommand error message single-sourced from a `SUBCOMMANDS` const.
- **F10** the env-leak test uses ambient `HOME` instead of mutating process-global env
  (racy under parallel tests; `unsafe` in edition 2024).
- **F26/F27** Cargo metadata (license/repo/keywords/categories/rust-version),
  `.gitignore` hardening, `.gitattributes` (`eol=lf` so a Windows clone can't change a
  committed script's `code_hash`; ledger/evidence marked binary).
- **F8, scoped deliberately:** full `clippy::pedantic` is **130 warnings**, dominated by
  `must_use_candidate`/`missing_errors_doc`/`module_name_repetitions` noise — enabling
  it under CI's `-D warnings` would force a disproportionate 130-fix diff. Adopted a
  *curated* high-signal set via Cargo's `[lints.clippy]` table (single source, all
  targets): `uninlined_format_args`, `redundant_closure_for_method_calls`,
  `semicolon_if_nothing_returned`, `unnested_or_patterns`, `cast_lossless`. Full
  pedantic left as a deliberate non-goal.
**Zero frozen surfaces changed** (F5/F7 are byte-preserving; rot guard `t15b` verifies).

## D27 — Capability-gated tests must be `#[ignore]`, not silent skips (s16, F11)
**Context:** Three headline features — network namespaces, read-only mounts, real
Hypothesis property testing — gated on host capability and `return`ed silently when
absent. On ubuntu CI (no CAP_SYS_ADMIN, no hypothesis) all three reported PASS *without
executing*; a regression in the runner's isolation setup would ship green.
**Decision:** The honesty doctrine (D14 "silence never reads as success", D19 "success
never reads as failure") applied to test reporting: a not-run test must read as
*ignored*, never *passed*. The three tests are now `#[ignore = "…"]`; a privileged CI
job (`--privileged --cap-add=SYS_ADMIN`, pip-installs hypothesis) runs them for real via
`cargo test -- --ignored`. Verified live here with `--include-ignored`.
**Scoping:** F9 (`tests/common/` consolidation), F13 (Rust proptest), F14 (`parse_args`
extraction) deferred as coverage-*depth*, not correctness-*masking* — F11 was the only
item that let a real regression ship green. Added F12 parser edge-case tests where
cheapest (`tap::parse_libtest`, `Hash::from_str`).

## D28 — Fix the O(N²) sidecar appends (F1 substance); share the primitive, not the ledger (s17)
**Context:** F1 flagged the hash-chained-ledger pattern duplicated 4× with an O(N²)
re-read-per-append bug. Re-audit at s17: `judge.rs` was already fixed (s9 `JudgmentWriter`
is open-once), so the live bug was only `mutation.rs`/`fuzz.rs` — each called `read_*`
(read + chain-verify the whole file) on every append.
**Decision:** Fix the bug the proven way — open-once `MutationWriter`/`FuzzWriter` that
read the tail at open and keep `(last_hash, next_seq)` in memory (O(1) per append). Share
the *bookkeeping primitive* (`chained::ChainState` + `append_ndjson_line`), **not** a
generic-over-entry-type ledger: the four entry layouts differ and one (confirmations) is
freeze-locked, so a shared trait would carry more machinery than a 4-instance pattern
earns. Byte layouts are unchanged — each writer keeps its exact `canonical` bytes.
- **F6:** shared `cache::read_cache<T>` for the four cache sites; fixes fuzz's
  `read_to_string(...).unwrap_or_default()` (which hid permission/I-O errors as `""`).
  A genuine miss (`NotFound`) is silent; corruption is surfaced on stderr, never
  conflated (the D14/D19 honesty doctrine, applied to the cache).
- **F4:** typed `Spawn`/`Malformed`/`ChainBroken` variants in `MutationError`/`FuzzError`
  (was: spawn reported as `Io` on the program path; malformed/broken as `ConfigInvalid`),
  matching `LedgerError`'s diagnostics with line/seq detail.
**Deferred (documented):** the fully-generic `HashChainedLedger<T>` extraction — the
maintainability-only half of F1 — as disproportionate machinery for this pattern.
**Coverage note:** sidecar layouts aren't covered by the durable-ledger rot guard, so a
2-entry multi-append chain-verify assertion was added to the two-unit mutation test as
their witness. 127 pass / 3 ignored; byte-preserving.

## D29 — Security hardening: validate at the source, contain defensively, document the trust boundary (s20)
**Context:** The refactoring plan's security cluster (F16–F18, F21). Re-audit found the
plan's premises were partly stale:
- **F16** claimed the judge's `critique_ref` was attacker-supplied and joined to the
  state dir without validation. In fact `critique_ref` is *engine-generated*
  (`ledger/critiques/<64-hex cell_key>/N.md`) and cannot traverse. The join was not a
  live vulnerability.
- **F18** (unit `id` reaches path construction) *was* real: `id` is author-controlled and
  flows into mutation work-dir paths (`mutation.rs` `id.replace('/', "_")`), so a crafted
  `id` (`../`, separators, leading dot, control chars) could traverse or confuse the FS.
**Decision:** Apply defense at the *source* and containment at the *use site*, and record
the premise corrections so the divergence from the plan is auditable.
- **F18 (real fix):** `validate_unit_id` at manifest load time — reject empty, path
  separators, `..`, leading dot, control/whitespace chars. Dotted namespacing
  (`u.parser.tokenize`) stays legal. Rejecting at parse time keeps the error at its cause,
  not at a downstream DAG/FS symptom (the same doctrine as the existing self-dep check).
- **F16 (defensive, premise corrected):** `safe_state_path(state_dir, ref)` rejects
  absolute paths and any non-`Normal` component, used at the repair-loop join. It never
  rejects a value the engine produces today — it is a guard for the day a judgment is
  loaded from disk (hence attacker-influenceable) rather than computed in-process. The
  stale "live vuln" framing is corrected here.
- **F17 / F21 (document the boundary):** Safety/trust doc-comments on `run_repair` (spawns
  an operator-authored command at test trust level; nothing attacker-controlled reaches
  argv/env; not sandboxed by design — the next det round re-runs under the sandbox) and on
  the `unsafe fn make_root_readonly` (caller must already be in a fresh private mount
  namespace; the precondition is a contract the sandbox upholds, uncheckable here).
**Coverage:** new tests — id-traversal rejection (manifest) and `safe_state_path`
accept/reject (judge unit test). 129 pass / 3 ignored; zero frozen surfaces touched.

## D30 — Enum-ify the manifest scope keys: make an unknown scope unrepresentable, not merely rejected (s18, F2)
**Context:** `Manifest.tests` was `BTreeMap<String, TestSpec>`, validated at load against a
`VALID_SCOPES` string slice; `round.rs` then re-looked-up each `CellScope` by
`scope.as_str()`. Two representations of the same closed set (the string map and the
`CellScope` enum) with a hand-rolled bridge between them.
**Decision:** Type the map key as `CellScope` directly (`BTreeMap<CellScope, TestSpec>`).
`CellScope` gains `Serialize`/`Deserialize` with `#[serde(rename_all = "lowercase")]`, so
`[tests.unit|direct|closure|e2e]` parse straight into the domain type and `[tests.galactic]`
is a **parse error** from the deserializer — the `VALID_SCOPES` const and its validation
loop are deleted. `round.rs` drops the string round-trip (`tests.get(&scope)`).
**Freeze safety (the load-bearing check):** this touches `hash.rs`, a frozen module, but
changes *nothing hashed*. Only `scope as u8` ever enters a `cell_key` (`compute_cell_key`),
and the four discriminants (0..3) are unchanged; the string form is manifest sugar that
never reaches a key. The rot guard (`t15b_durable`) passing is the witness that the durable
ledger still verifies byte-for-byte.
**Coverage:** a manifest-layer test pins the new behavior (unknown scope fails to parse; the
four valid scopes load); the existing `t5b` load-rejection test still holds. 130 pass / 3
ignored; clippy -D warnings + fmt clean.
**Note:** kept the `BTreeMap<CellScope, _>` shape rather than a struct-of-`Option`s — the map
preserves every call site's `.get`/`.values`/`.is_empty` semantics for a smaller, obviously
byte-neutral diff, and `toml` deserializes enum map keys cleanly.

## D31 — Decompose the three longest functions along their natural seams (s19, F3)
**Context:** F3 flagged over-long functions. The three worst were `full_audit` (153 lines),
`run_mutation` (139), and `run_round` (131) — each a coherent algorithm, but each folding
several independent concerns into one body where a reader has to hold all of them at once.
**Decision:** Extract along the seams the code already has, behavior-preserving (no byte,
no ledger, no key change — pure readability/testability):
- **`full_audit`** → four phase helpers over one shared `AuditReport`: `audit_confirmations`
  (returns the verified entries the later phases reuse), `audit_roots`, `audit_sidecar_chains`
  (judgments/mutations/fuzz), `audit_evidence`. The top function is now a four-line
  narrative of the audit; each surface is independently readable and testable.
- **`run_round`** → `resolve_cell`, lifting the per-cell frontier decision (gated-skip /
  cache-hit / hermetic-run-store-cache, incl. quarantine) out of the tier-gating loop. The
  loop now reads as *gating*; the cell economics read as *one function*.
- **`run_mutation`** → a borrowing `MutationRun<'a>` context struct (the run-wide invariants:
  units_dir, paths, config, toolchain, seed, baseline_root, mutator_hash) with `score_unit`
  (per-unit tally → `UnitScore`) and `evaluate_mutant` (one mutant → a `MutantOutcome`
  enum: Killed / Survived / Skipped). The context struct is deliberate: passing the seven
  invariants as arguments would trip `clippy::too_many_arguments` and re-thread them at
  every call — the struct names the run once and the methods stay small.
**Deferred (documented):** `run_cell` (128, runner.rs) — the fork/exec + namespace sandbox.
Its privileged paths are `#[ignore]`-gated (run only in the privileged CI job), so a
decomposition there carries more regression risk per line than the three above; left for a
focused pass rather than bundled with lower-risk work.
**Verification:** 130 pass / 3 ignored; clippy -D warnings + fmt clean. The behavior-fixing
tests (t16_audit, t12_mutation, t5*/round) are the witnesses that nothing moved but the
seams.

## D32 — Perf cleanup + documentation currency, re-derived from the code (s21)
**Context:** The external refactoring plan's last two clusters were performance and
docs/archival. That plan was attached in chat and never committed, so rather than guess at
its F-numbers, this sprint re-derives the work from the code directly (the same measure-first
method used for the F3 decomposition) and records it honestly as re-derived.
**Perf — single, defensible structural fix:**
- `audit_evidence` enumerated the evidence directory **twice**: once to hash-check every
  file, then again to collect the set of stored content-address stems for the
  "missing evidence" note. Folded into **one** `read_dir` pass — the stem set is now built
  during the hash-check loop. Behavior is identical (the second pass only ever read
  `file_stem`, which the first pass already extracts; unreadable files still contribute
  their stem, matching the old set exactly). This is cold-path I/O, not a hot loop — chosen
  because it is a clean win with zero behavior risk. The tempting `by_round` clone in
  `audit_roots` was **left alone**: eliminating it would force `RootRecord::from_entries`
  to take `&[&LedgerEntry]`, rippling the ledger API for negligible gain on the same cold
  audit path. No hot-path change was manufactured — the engine's real performance story is
  the frontier/caching economics (only changed cells re-run), which is the core design, not
  something a micro-opt would improve.
**Docs — currency:** the README's "Sprint-loop state" and "Status" had frozen at
**D1–D8 / s0–s10 / 109 tests**. Brought current: D1–D31, sprints s0–s20 (condensed, grouped
kernel / post-freeze / refactoring pass), 130 tests (+3 ignored), and the fuzz tier (T13/D24)
which the status blurb had never mentioned. A repo offered as a template must not describe a
past version of itself.
**Verification:** 130 pass / 3 ignored; clippy -D warnings + fmt clean; t16_audit is the
witness that the single-pass audit is behavior-identical.

## D33 — Decompose run_cell, the function D31 deferred (s22, F3 follow-up)
**Context:** D31 decomposed the three longest functions but deliberately left `run_cell`
(128 lines) — the `unsafe` fork/exec + namespace sandbox — for a focused pass, since its
privileged branches only run in the gated CI job and carry more regression risk per line.
This is that pass.
**Decision:** Split along the three phases the function already had, behavior-preserving:
- **`build_cell_command`** — cleared env + hygiene set + declared vars + seed, piped stdio,
  and (unix) the process-group + sandbox install. Returns the configured `Command`.
- **`install_sandbox`** (unix) — the `process_group(0)` + the `pre_exec` closure (memory
  cap, netns unshare, read-only mount, fail-closed). Now carries a proper `# Safety`
  doc-comment spelling out the post-fork/pre-exec contract: async-signal-safe libc only,
  heap-free captures, fail-closed — the trust-boundary documentation the s20 F17/F21 pass
  established as doctrine, now applied to the one remaining under-documented `unsafe`.
- **`wait_with_envelope`** — the try-wait/timeout/SIGKILL-the-group loop; returns
  `(exit_status, timed_out)`.
`run_cell` (128 → 55) now reads as: resolve program → build command → spawn → drain pipes →
wait → classify → evidence.
**Risk management:** no byte/behavior change. The `install_sandbox` branches are covered
across *both* CI jobs — the memory-cap branch by the non-ignored `t3b` mem-cap test (the
normal `test` job; `setrlimit` needs no privilege), the netns and mount branches by the
`#[ignore]`-gated `t3b`/`t3c` tests in the privileged job. 130 pass / 3 ignored; clippy
-D warnings + fmt clean.
**Milestone:** with this, the external refactoring plan (F1–F42) is fully worked through —
substance done, stale premises corrected in the log, maintainability-only extras
consciously deferred where documented.

## D34 — T8b: make the `proved` tier live via CBMC (Kani's engine), through authorized egress (s23)
**Context:** The `proved` guarantee tier (D17, §7.2) was a recorded *declaration* with no
committed unit that actually verified something over its whole input space. T8b makes it
live. The plan named Kani (the Rust-native model checker); provisioning it hit a hard wall.
**The egress finding (recorded so it isn't rediscovered):** `cargo kani setup` downloads a
release bundle from the `model-checking/kani` GitHub repo, and this session's egress policy
scopes GitHub to `crussella0129/array-test` only — every other repo path returns 403
(`api.github.com/repos/model-checking/kani/...` and the release-download path both 403;
the scoped repo returns 200). That is the same deliberate repo-scope restriction that
governs the session, not a fixable config issue, so routing around it via the reachable
`objects`/`raw` CDNs was rejected.
**Decision:** Use **CBMC** — the C bounded model checker that Kani wraps — installed from
the Ubuntu archive (authorized egress). This gives the `proved` tier a genuine, live
symbolic proof through an allowed channel, and CBMC being Kani's own engine makes it a
faithful realization, not a downgrade.
- Committed `examples/proved-cbmc/units/nibble-roundtrip/`: a C harness with a
  nondeterministic byte (CBMC checks all 256 values at once) proving the hex-nibble
  round-trip identity + hex-digit validity — the invariant behind `Hash::hex`. A wrapper
  emits deterministic TAP (CBMC's timing-bearing output is discarded so the run-twice
  meta-check sees byte-identical evidence). Manifest declares `guarantee = "proved"`.
- `tests/t8b_proved.rs`: a non-ignored engine-plumbing test (proved level records without
  any prover, runs everywhere) + two `#[ignore]` + self-skip tests (D27) — the real proof
  passes and records `Guarantee::Proved`; a **falsified** harness (a refutable assertion)
  turns the round red, proving the proof proves something. Validated live this session with
  CBMC installed (VERIFICATION SUCCESSFUL; the bug case VERIFICATION FAILED → red).
- CI: the `privileged-tests` job now `apt-get install`s `cbmc` and runs it via `--ignored`,
  so the live proof runs in CI — not theater.
**Honesty:** this is the D14/D19/D27 doctrine applied to a headline claim — the `proved`
tier now *demonstrably* verifies over the whole input space in CI, and where the prover is
absent the tests read as *ignored*, never falsely passed. 131 pass / 5 ignored normally;
136 with CBMC. Kani (Rust path) remains a future addition if its bundle host is ever
authorized; the tier is prover-agnostic by design.

## D35 — T14: the sprint-loops Test-phase adapter, kept off the agnostic core (s24)
**Context:** array-test was built to power the Test phase of sprint-loops, but D11 makes the
core consumer-agnostic — it must never reference sprint-loops. T14 is the shim that wires
the two together without violating that.
**Decision:** A thin, optional adapter at `adapters/sprint-loops/` — a POSIX-sh Test-phase
entrypoint (`array-test-phase.sh`) plus its README — that runs one array-test round over a
sprint's units and gates the sprint on a green, re-verified root. It touches no engine
code; the core stays agnostic (the adapter depends on array-test, never the reverse). The
mapping it realizes: a sprint's deliverable → units; the test-plan's ACs → cells; the Test
verdict → a green root; the persistent `.array-test/state/` → the project's durable,
independently re-verifiable test memory. Because state persists across sprints, each Test
phase is incremental (unchanged units reused, byte-identical root) — the founding schema's
down/out/backwards array made economical by content addressing.
**Verified here:** `tests/t14_sprint_loops_adapter.rs` drives the script as a black box —
green project → exit 0 + a `test-record.md` carrying the root; a broken unit → exit 1, RED
record; a missing binary → exit 2. Ran the phase live against the quickstart units (green,
then frontier-reuse on re-run, then red after a break).
**The `array-test-fork` limitation (reported, not worked around):** the user asked for this
on the sprint-loops side, in a fork named `array-test-fork`. This session's GitHub token is
scoped to `crussella0129/array-test`; `create_repository`/`fork_repository` for anything
else return `403 Resource not accessible by integration` (the same scope wall that blocked
the Kani bundle in D34). So the adapter is delivered in-scope, self-contained, and designed
to be copied straight into that fork — the fork's creation is the user's to do (or a
session with broader GitHub scope). Not circumvented.

## D36 — Containerize the evidence-producing environment; prove the shipped image, not the build (s25, C1–C3)
**Context:** The usability assessment found array-test is Linux-first (the netns /
`mount_setattr` / `RLIMIT_AS` sandbox) and its strongest tiers need provisioned tools
(CBMC, Hypothesis). The user's containerization instinct fits the project's own doctrine:
an image pins the *runner's* environment the way `toolchain.lock` → `toolchain_hash` pins
the *tested* toolchain — content addressing extended to the evidence-producing runtime.
**Decision:**
- **C1:** a multi-stage `Dockerfile` — `rust:1-slim-trixie` builder → `debian:trixie-slim`
  runtime carrying only the release binary + `cbmc` + `python3-hypothesis` + the T14 shim +
  the committed example workspaces. Same Debian release in both stages (glibc match); no
  Rust toolchain ships. `proved`/`property` are live by default in the image.
- **C2:** two documented run modes (README "Container image"): plain run = EnvOnly,
  `--privileged`/`--cap-add=SYS_ADMIN` = full sandbox — the isolation level is recorded
  honestly per confirmation either way. Framing correction recorded: the *image* is
  immutable; the running *container* is not without `--read-only` — what the image buys is
  reproducibility of the environment, not runtime tamper-proofing.
- **C3:** a `docker` CI job that proves the **shipped artifact**, not the build: quickstart
  and proved-CBMC rounds green *inside* the image, `import hypothesis` succeeds, the T14
  shim runs, and — the non-theater witness — under `--privileged` the ledger records
  `net_isolated`, so the sandbox demonstrably applied inside this image. (Green rounds
  alone could not distinguish EnvOnly from sandboxed; the ledger grep can, because D16
  records the level per confirmation. The honesty doctrine made this assertion possible.)
- **Positioning (per D11):** one distribution channel among several — binary,
  `cargo install`, and the library API stay first-class. C4 (registry publication by
  digest) and C5 (genesis-ritual parity) remain on the backlog.
- **Polish folded in:** `--version`/`-V`/`version` (was: exit-2 error), single-sourced from
  `CARGO_PKG_VERSION`, covered in t11_cli; it doubles as the image's smoke command.
**Verification:** local — 135 pass / 5 ignored, clippy -D warnings + fmt clean. The
Dockerfile and docker job cannot run in this session (no docker daemon); the PR's `docker`
CI job is the verifier, per the established CI-green-before-merge rule.
