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

### 1.3 Integration DAG
Edges = declared `deps`. The DAG is the **only** source of integration scope — we never
test arbitrary unit pairings, only declared compositions. This is the single most
important lever against combinatorial blow-up (§4).

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

A **confirmation** is the recorded result for a `cell_key`:

```
confirmation = { cell_key, status: PASS|FAIL, evidence_hash, ts, signer }
```

**Invariant:** a given `cell_key` maps to exactly one status, forever. If the code,
deps, test, fixture, seed, or toolchain change, the key changes → it's a *new* cell that
must be confirmed. If none changed, the previous ✓ is reused with zero execution. This is
the schema's "✓ at each step", made cacheable.

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

## 4. Taming the exponential — the explicit levers

| Source of blow-up | Lever | Result |
|---|---|---|
| All-pairs / all-subsets integration | Only DAG-declared edges produce integration cells | Integration cells = O(edges), not O(2^units) |
| Re-running full history every round | Content-addressed memoization of confirmations | Round cost ∝ changed frontier |
| A change "touching everything" | Reverse-dep impact closure bounds the blast radius | Only genuinely-affected cells re-run |
| Deep closures re-executed at every scope | Scope ladder UNIT→DIRECT→CLOSURE→E2E with fail-fast | Cheap scopes gate expensive ones |
| Flaky/non-deterministic re-keys | Hermetic execution (§5) freezes the key inputs | Stable keys → high cache hit rate |

If `confidence < 0.5` (see sprint-loop throttle), additionally cap the frontier: defer
`E2E`/`CLOSURE` of low-risk untouched subtrees to a nightly full re-key.

---

## 5. Determinism (the precondition for everything above)

A cell is only cacheable if it's reproducible. Required:

- **Pinned seeds** for all RNG (incl. property-based generators).
- **Frozen clock** — injected time source, never `now()`.
- **No ambient I/O** — network blocked; filesystem is a content-addressed fixture store.
- **Pinned toolchain** — compiler/runtime/deps lockfile hashed into the key.
- **Ordered iteration** — no hash-set ordering leaks into output.
- **Single-writer ledger** — confirmations appended in topological order.

Determinism check (meta-test): run any cell twice; `evidence_hash` must match. A cell that
fails this is *quarantined* (cannot enter the cache) until made hermetic.

---

## 6. Provability — what we actually claim

Two distinct, deliverable guarantees:

### 6.1 Proof of execution (auditable)
The confirmation ledger is **append-only and hash-chained**:

```
ledger_entry_n.prev = H(ledger_entry_{n-1})
```

The regression array's **Merkle root** commits to the multiset of `{cell_key → status}`.
Therefore:

> A single green root hash certifies: *for exactly this code (these `code_hash`es), this
> toolchain, these seeds, every cell in the array has a recorded PASS.* Anyone can
> recompute the root from the ledger and verify it — no trust in the runner required.

This is the "in real time / history" record from the drawing, made tamper-evident.

### 6.2 Universal correctness (stronger than examples)
- **Contracts** (pre/post/invariants) are checked on every cell that exercises a unit.
- **Property-based tests** assert `∀ x ∈ domain. P(x)` via generation + shrinking — a
  passing property is a claim over the whole domain, not one example.
- **Optional formal tier**: for designated critical units, invariants are discharged by a
  model checker / SMT (e.g. encode the contract, prove no counterexample). These units get
  a `proof_hash` in their confirmation.

We are explicit that 6.1 is always on; 6.2 scales with effort and 6.1's audit trail records
*which* guarantee level each cell achieved (`example | property | proved`).

---

## 7. On-disk layout (the state machine)

```
array/
  units/<id>/...                      # §1.1
  dag.json                            # resolved integration DAG (computed)
  ledger/
    confirmations.ndjson              # append-only, hash-chained (§6.1)
    roots/R<k>.json                   # certified root per round
  cache/<cell_key>.json               # memoized confirmations (✓ reuse)
```

The CLI is a pure function of this tree: `array-test run` reads `units/` + `ledger/`,
computes the frontier, executes, appends confirmations, writes a new root. Re-running with
no changes is a no-op that re-verifies the existing green root.

---

## 8. How sprints drive the array (loop integration)

Each sprint = one `R_k`:

1. **Research** — read prior `roots/R(k-1).json` + any `failure-report.md`.
2. **Plan** — `build-plan.md` lists new/changed units; `test-plan.md` lists the cells they
   introduce and the expected frontier.
3. **Build** — implement units; `code_hash`es change.
4. **Test** — run `R_k`; only the frontier executes; append confirmations; compute root.
5. **Loop** — green root → close sprint, feed root forward (the `R6 → current sprint`
   arrow). Red root → write `failure-report.md`, drop confidence, shrink next frontier.

The array and the sprint loop are the same machine viewed at two timescales: the array is
the data structure, the sprint loop is its update protocol.

---

## 9. Build order (forward reference to the backlog)

1. Content-addressing + manifest/contract schema (`code_hash`, `cell_key`).
2. Integration DAG resolver + impact (reverse-dep) closure.
3. Hermetic cell runner + determinism meta-check.
4. Confirmation ledger (append-only, hash-chained) + Merkle root.
5. Frontier selection (memoized; cache reuse).
6. Property-based + contract tiers; optional formal tier.
7. CLI + sprint-loop wiring.

See `agent-tasks/agent-tasks.md` and `sprints/s0/` for the concrete first sprint.
