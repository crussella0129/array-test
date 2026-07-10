# Architecture — `array-test`

A deterministic, code-based, provable regression system for agentic software
construction. Derived directly from `SCHEMA-ANALYSIS.md`.

> **Thesis.** The regression "array" in the schema is a **Merkle DAG of confirmations**.
> Each test cell is keyed by the hash of everything that can affect it; a cell only re-runs
> when its key changes; the root hash of the DAG is a single, verifiable certificate that
> "all confirmations hold for exactly this code." This converts the schema's exponential
> lattice into work proportional to the *changed frontier*, and gives us "provable" in a
> sense we can actually ship.

---

## 1. Objects

### 1.1 Unit (`U`)
The atomic deliverable produced in a sprint.

```
unit/
  <unit-id>/
    manifest.toml      # identity, deps, contract ref, version
    src/...            # implementation
    contract.toml      # typed I/O + invariants + property specs
    tests/             # deterministic unit suite (seeded)
```

`manifest.toml` (normative shape):

```toml
id        = "u.parser.tokenize"     # stable, globally unique
sprint    = 3                        # provenance: which sprint created/last-changed it
version   = "1.4.0"
deps      = ["u.io.read", "u.errors"]   # direct integration edges (DAG, no cycles)
code_hash = "blake3:…"               # content hash of src/ + contract (computed, not authored)
```

A unit is **content-addressed**: `code_hash = H(src/ ‖ contract.toml)`. Identity is the
`id`; the `code_hash` is what changes when the unit changes.

### 1.2 Contract
The unit's promise. This is where "provable" starts — claims are *universal*, not
example-based.

```toml
[io]
input  = "Bytes"
output = "List[Token]"

[invariants]            # checked on every cell that exercises this unit
pre  = ["len(input) >= 0"]
post = ["all(t.span.end <= len(input) for t in output)"]

[properties]            # ∀-quantified, run with generated inputs (property-based testing)
roundtrip  = "detokenize(tokenize(x)) == normalize(x)"
monotonic  = "len(tokenize(x)) <= len(x)"
```

When a unit has no exact oracle (common for agent-generated code whose spec is prose),
prefer **metamorphic relations** in `[properties]` — relations between outputs rather
than exact outputs, e.g. `tokenize(a ++ b)` versus `tokenize(a)`/`tokenize(b)` (D10).
A Phase-J judge reviewing an oracle-less contract should ask for them.

**Authoring convention:** individual test cases (`tests/`) are written in a
`given / should / actual / expected` shape — the convention
[riteway](https://github.com/crussella0129/riteway) organizes its API around — e.g. in
Rust:

```rust
#[test]
fn tokenize_empty_input() {
    // given: an empty byte string
    // should: return an empty token list
    let actual = tokenize(&[]);
    let expected: Vec<Token> = vec![];
    assert_eq!(actual, expected);
}
```

This is adopted deliberately, not incidentally (see D6 in `decisions.md`): the
`given/should/actual/expected` shape forces every test to answer the same five questions
§7 needs answered anyway (unit under test, expected behavior, actual output, expected
output, reproduction). The shape is language-agnostic; what's load-bearing is the output —
**TAP** (Test Anything Protocol) — which becomes the raw `evidence` a cell's confirmation
hashes over (§2), instead of a bespoke evidence format. Per D8, the engine itself is Rust;
riteway (JS) is an optional adapter for units written in JS, not a dependency of the core.

### 1.3 Integration DAG
Edges = declared `deps`. The DAG is the **only** source of integration scope — we never
test arbitrary unit pairings, only declared compositions. This is the single most
important lever against combinatorial blow-up (§5).

- **Down** = traverse a unit's dependency closure (compose with what it's built on).
- **Backwards** = traverse a unit's *reverse*-dependency closure (who is affected when it
  changes).

### 1.4 Regression array
A set of **cells**. A cell is one confirmation at one integration scope:

```
cell = (target_unit, scope)
scope ∈ { UNIT, DIRECT, CLOSURE, E2E }      # the "down" axis
```

- `UNIT`   — target in isolation (mocks at every dep boundary).
- `DIRECT` — target + its direct deps.
- `CLOSURE`— target + full transitive dependency closure.
- `E2E`    — root-level entrypoints exercising real wiring.

The array is the cross product `{units} × {scopes}` **pruned to the DAG** — i.e. a
`CLOSURE` cell only exists where a closure is meaningful; an `E2E` cell only at declared
entrypoints.

---

## 2. The cell key (why nothing re-runs unnecessarily)

Every cell has a **content-addressed key**:

```
cell_key = H(
    target.code_hash
  ‖ H(scope deps' code_hashes, in DAG order)   # the "down" closure
  ‖ test_def_hash                               # the test/property code itself
  ‖ fixtures_hash                               # hermetic inputs
  ‖ seed                                        # pinned RNG seed
  ‖ toolchain_hash                              # compiler/runtime/lockfile versions
)
```

All hashes are **domain-separated** (D9): each derives under a frozen, versioned BLAKE3
`derive_key` context (`array-test/v1/code-hash`, `array-test/v1/cell-key`, …) with
RFC 6962-style leaf/node role prefixes, so no hash produced for one purpose can ever be
presented as another (the Merkle second-preimage class of confusion). Paths inside
`code_hash` are normalized to `/`-joined UTF-8 and sorted as strings; non-UTF-8 names and
symlinks are rejected — a key must mean the same bytes on every platform, or §7's claims
quietly become platform-scoped.

A **confirmation** is the recorded result for a `cell_key`:

```
confirmation = {
  cell_key,
  det_status: PASS|FAIL,      # §3: the hermetic, reproducible run
  evidence_hash,               # hash of the raw TAP output (§1.2)
  judge:        null | { rating, threshold, runs, judge_hash, critique_ref },  # §4
  status:       det_status == PASS AND (judge == null OR judge.rating >= judge.threshold),
  ts, signer
}
```

**Invariant:** a given `cell_key` maps to exactly one `det_status`, forever — that part is
strictly reproducible and is what the Merkle root (§7) commits to. `judge` is recorded
alongside but kept out of the root's reproducibility claim, since a judge's rating is not
guaranteed bit-identical run to run (§4). If the code, deps, test, fixture, seed, or
toolchain change, `cell_key` changes → it's a *new* cell that must be confirmed. If none
changed, the previous ✓ is reused with zero execution. This is the schema's "✓ at each
step", made cacheable.

---

## 3. The regression run (`R_k`) = wavefront over the changed frontier

A regression round is **not** "run everything." It is:

```
Rk(changeset):
  1. dirty   = units whose code_hash changed since R(k-1)
  2. impact  = reverse-dependency closure of dirty        # "backwards"
  3. frontier= { cells whose cell_key changed }           # = cells touching impact,
                                                           #   across scopes "out" & "down"
  4. for cell in topological_order(frontier):             # deterministic order
        run cell hermetically -> status                   # confirmation at each step
        if FAIL: record, and (policy) halt-or-continue
  5. recompute Merkle root over ALL cells
        (frontier = freshly run; everything else = reused ✓)
  6. root green?  -> Rk certified.  feed root to current sprint (loop back)
```

- **out**: new units in the current sprint enter as new cells.
- **down**: each cell composes along its scope's dependency closure.
- **backwards**: step 2 — a change re-confirms ancestors/dependents transitively.
- **loop back**: step 6 — `R_k`'s green root is the gate the current sprint reads
  (`R6 → current sprint` in the drawing).

Cost of `Rk` ∝ `|frontier|`, **not** `|all cells|`. That is the answer to "exponential
growth": history is paid for once and amortized via reuse.

---

## 4. The two-phase confirmation gate (deterministic AND judged)

Passing tests is necessary but not sufficient: a cell can be `det_status = PASS` and still
not do what the spec meant. So a cell is only fully **confirmed** if it clears two gates in
series — not two independently-recorded tiers, a genuine `AND`:

```
confirmed(cell)  ⇔  det_status == PASS   AND   judge.rating >= judge.threshold
```

### 4.1 Phase D — Deterministic (required, reproducible)
Exactly §3: the hermetic cell runs, produces `det_status`. This gate is objective,
cacheable, and is what the Merkle root certifies. **A Phase D failure is handled entirely
by the existing regression machinery** (§3 step 4, impact re-confirmation) — nothing new
here, and Phase J is never entered.

### 4.2 Phase J — Judged (required, non-reproducible-by-nature)
Only entered once Phase D = PASS. Modelled on
[`riteway ai`](https://github.com/crussella0129/riteway): an independent judge agent reviews
the unit — implementation, contract, and the Phase-D evidence (TAP output) — against the
spec, across `judge.runs` passes, and must clear `judge.threshold` (e.g. 8/10 runs rate it
faithful to spec). The judge is checking something Phase D structurally cannot: *does this
correctly implement the intent*, not merely *does it not crash / does it satisfy the
assertions someone wrote*.

Because an LLM judge is not bit-deterministic, Phase J is **not** folded into the Merkle
root's reproducibility guarantee (§7 stays strictly about Phase D). Instead every judge
verdict is itself recorded in the append-only ledger — `judge_hash` pins the judge
model/version/prompt, `critique_ref` points at the full transcript — so the *verdict is
auditable* even though *re-running the judge* isn't guaranteed to reproduce it bit-for-bit.
This is the honest boundary: Phase D gives reproducible proof, Phase J gives an audited,
threshold-smoothed expert opinion.

Phase J also owns **golden updates** (D10): when a test's expected output changes, the
new golden routes through the judge as a semantic event ("the promise changed — is that
right?"), never an auto-accept. This is the standing defense against golden rot.

### 4.3 Judge failure → the repair micro-loop
If Phase D passes but Phase J fails, that is **not** a sprint-level failure. It triggers a
micro-loop scoped to exactly this cell's unit:

```
repair(unit, critique):
  1. Plan  — a patch plan derived from the judge's critique (scope: this unit only)
  2. Build — patch the unit  → new code_hash → new cell_key
  3. Test  — Phase D on the new cell_key, then Phase J again
  4. converged?  -> cell confirmed, resume the enclosing R_k
     not converged, retries < budget -> loop to 1 with the new critique
     retries exhausted -> escalate: sprint-level failure-report.md, drop confidence
```

This is the schema's "backwards" arrow at unit scale: instead of only re-confirming
*ancestors* when a dependency changes (§3 step 2), a judged rejection folds back into a
tight, bounded Plan→Build→Test loop *around the single unit*, and only escalates to the
full sprint loop (`sprints/sN/failure-report.md`) if the micro-loop can't converge within
its retry budget. A converged repair produces an ordinary new `cell_key`, so it re-enters
the regular frontier — the micro-loop is a local fixed-point search, not a separate code
path.

---

## 5. Taming the exponential — the explicit levers

| Source of blow-up | Lever | Result |
|---|---|---|
| All-pairs / all-subsets integration | Only DAG-declared edges produce integration cells | Integration cells = O(edges), not O(2^units) |
| Re-running full history every round | Content-addressed memoization of confirmations | Round cost ∝ changed frontier |
| A change "touching everything" | Reverse-dep impact closure bounds the blast radius | Only genuinely-affected cells re-run |
| Deep closures re-executed at every scope | Scope ladder UNIT→DIRECT→CLOSURE→E2E with fail-fast | Cheap scopes gate expensive ones |
| Flaky/non-deterministic re-keys | Hermetic execution (§6) freezes the key inputs | Stable keys → high cache hit rate |
| Judge-rejected units needing global re-plan | Repair micro-loop (§4.3) is scoped to one unit | Fix cost ∝ one unit, not the sprint |

If `confidence < 0.5` (see sprint-loop throttle), additionally cap the frontier: defer
`E2E`/`CLOSURE` of low-risk untouched subtrees to a nightly full re-key.

---

## 6. Determinism (the precondition for everything above)

A cell is only cacheable if it's reproducible. Required:

- **Pinned seeds** for all RNG (incl. property-based generators).
- **Frozen clock** — injected time source, never `now()`.
- **No ambient I/O** — network blocked; filesystem is a content-addressed fixture store.
  *(v1 status, D12/R-g: the runner enforces env hygiene + the meta-check below; actual
  network/memory isolation is T3b. Until then this bullet is an aspiration the
  meta-check polices, not a sandbox guarantee — and the docs say so on purpose.)*
- **Pinned toolchain** — compiler/runtime/deps lockfile hashed into the key.
- **Ordered iteration** — no hash-set ordering leaks into output.
- **Single-writer ledger** — confirmations appended in topological order.

Determinism check (meta-test): run any cell twice; `evidence_hash` must match. A cell that
fails this is *quarantined* (cannot enter the cache) until made hermetic. Quarantine is
**visible ledger state** — a quarantined cell is a red mark with a recorded reason, never
a silent skip; quarantine must not become the place failures go to be forgotten (D10).

Additionally (D10), each scope in the ladder carries a **resource envelope**
(wall-clock/memory caps enforced by the runner, T3): a `UNIT` cell that outgrows its
envelope is an early signal it has quietly become an integration test.

---

## 7. Provability — what we actually claim

Three distinct, deliverable guarantees — deliberately kept separate so a probabilistic
layer never quietly weakens a reproducible one:

### 7.1 Proof of execution (auditable, reproducible — Phase D only)
The confirmation ledger is **append-only and hash-chained**:

```
ledger_entry_n.prev = H(ledger_entry_{n-1})
```

The regression array's **Merkle root** commits to the multiset of `{cell_key → det_status}`
— Phase D only, never Phase J. Therefore:

> A single green root hash certifies: *for exactly this code (these `code_hash`es), this
> toolchain, these seeds, every cell in the array has a recorded, reproducible PASS.*
> Anyone can recompute the root from the ledger and verify it — no trust in the runner
> required.

This is the "in real time / history" record from the drawing, made tamper-evident.

### 7.2 Universal correctness (stronger than examples — still Phase D)
- **Contracts** (pre/post/invariants) are checked on every cell that exercises a unit.
- **Property-based tests** assert `∀ x ∈ domain. P(x)` via generation + shrinking — a
  passing property is a claim over the whole domain, not one example.
- **Optional formal tier**: for designated critical units, invariants are discharged by a
  model checker / SMT (e.g. encode the contract, prove no counterexample). These units get
  a `proof_hash` in their confirmation.

We are explicit that 7.1 is always on; 7.2 scales with effort and 7.1's audit trail records
*which* guarantee level each cell achieved (`example | property | proved`).

### 7.3 Audited judgment (Phase J — recorded, not rooted)
The judge's verdict (§4.2) is appended to the ledger with its own hash chain
(`judge_hash`, `critique_ref`) so *what the judge said and why* is permanently auditable,
but it is deliberately **excluded** from the Merkle root in §7.1. A cell's practical gate
(§4) is `det root PASS AND judge PASS`; its *provability* claim is scoped precisely to the
deterministic root. This keeps "provable" honest: we never let a statistical opinion,
however well-audited, masquerade as a reproducible proof.

---

## 8. On-disk layout (the state machine)

```
<units-dir>/
  <unit-id>/...                       # §1.1 (manifest, contract, src, tests)
  judge.toml                          # optional: Phase J config (D17)
  toolchain.lock                      # optional: toolchain pin (D16)
<state-dir>/
  ledger/
    confirmations.ndjson              # append-only, hash-chained (§7.1) — det_status
    judgments.ndjson                  # append-only, hash-chained (§7.3) — judge verdicts
    critiques/<cell_key>/<n>.md        # judge transcripts (critique_ref targets)
    roots/R<k>.json                   # certified root per round (Phase D only)
    failures/R<k>-judgment.md         # escalation records (§4.3, D18)
  cache/<cell_key>.json               # memoized confirmations (✓ reuse)
  judge-cache/<cell_key>-<judge>.json # memoized verdicts (D17)
  evidence/<evidence_hash>.evidence   # content-addressed evidence store (re-hashable)
```

The CLI is a pure function of this tree: `array-test run` reads `units/` + `ledger/`,
computes the frontier, executes, appends confirmations (and judgments where Phase D
passed), writes a new root. Re-running with no changes is a no-op that re-verifies the
existing green root.

---

## 9. How sprints drive the array (loop integration)

Each sprint = one `R_k`:

1. **Research** — read prior `roots/R(k-1).json` + any `failure-report.md`.
2. **Plan** — `build-plan.md` lists new/changed units; `test-plan.md` lists the cells they
   introduce and the expected frontier.
3. **Build** — implement units; `code_hash`es change.
4. **Test** — run `R_k`; only the frontier executes Phase D, then Phase J (§4); judge
   failures resolve via the repair micro-loop (§4.3) before the sprint-level gate is
   evaluated; append confirmations + judgments; compute root.
5. **Loop** — green root (Phase D) **and** all frontier cells judge-confirmed → close
   sprint, feed root forward (the `R6 → current sprint` arrow). Red root, or a micro-loop
   that exhausted its retry budget → write `failure-report.md`, drop confidence, shrink
   next frontier.

The array and the sprint loop are the same machine viewed at two timescales: the array is
the data structure, the sprint loop is its update protocol. The repair micro-loop (§4.3) is
that same machine at a third, smaller timescale — one unit instead of one sprint.

### 9.1 Embedding contract (D11)
array-test is **library-first and consumer-agnostic**: the core never references
sprint-loops or any other harness. Integration happens against stable outputs only —
the all-PASS green gate, `roots/R<k>.json` certificates, the independently
re-verifiable `confirmations.ndjson` chain, and hash-committed TAP evidence. A
sprint-loops Test-phase shim (T14) maps those onto `test-report.md` /
`failure-report.md` and the phase exit condition, entirely on its own side of the
boundary. Any other application embeds the same way: call the library (or CLI), read
the certificate, optionally re-verify the chain yourself.

---

## 10. Build order (forward reference to the backlog)

1. Content-addressing + manifest/contract schema (`code_hash`, `cell_key`) — Rust.
2. Integration DAG resolver + impact (reverse-dep) closure — Rust (`petgraph`).
3. Hermetic cell runner + determinism meta-check — Rust.
4. Confirmation ledger (append-only, hash-chained) + Merkle root (Phase D) — Rust.
5. Frontier selection (memoized; cache reuse) — Rust.
6. TAP evidence adapter (§1.2) — `given/should/actual/expected` → TAP → `evidence_hash`.
   Native Rust test harness emits TAP directly; riteway remains available as an optional
   adapter for any unit written in JS.
7. Property-based tier via Python + Hypothesis (subprocess, TAP across the same evidence
   boundary as §6); contract tier (pre/post/invariants) in Rust; optional formal tier via
   Kani.
8. Judge gate (Phase J) + judgment ledger + repair micro-loop (§4).
9. CLI + sprint-loop wiring — Rust.

Toolchain locked in D8 (`decisions.md`): Rust core engine, Python/Hypothesis for property
tests. See `agent-tasks/agent-tasks.md` and `sprints/s1/` for the concrete next sprint.
