# Schema Analysis — Agentic Testing Array

> Interpretation of the hand-drawn schema (`current sprint`, `U` blocks, integration/
> regression array, `R1…R6`, "in real time / history"). This document is the source
> reading that the architecture is derived from. If the drawing and this text disagree,
> the drawing wins — flag it and we re-derive.

## 1. What the drawing shows

```
            Sprint 1        S2            S3            ...        Current sprint
            ┌──┐          ┌──┐┌──┐      ┌──┐┌──┐                    ┌──┐
   units →  │U │          │U ││U │      │U ││U │   ───loop back──▶  │U │
            └──┘          └──┘└──┘      └──┘└──┘                    └──┘
            ┌──┐┌──┐      ┌──┐┌──┐      ┌──┐                        ┌──┐
            │U ││U │      │U ││U │      │U │            ─────────▶  │U │
            └──┘└──┘      └──┘└──┘      └──┘                        └──┘
   ┌───────────────────────────────────────────────┐
 i │ R1 ✓ ─────────────────────────────────────────▶│  ──┐
 n │ R2 ✓ ─────────────────────────────────────────▶│    │
 t │ R3 ✓ ─────────────────────────────────────────▶│    │ travels
 e │ R4 ✓ ─────────────────────────────────────────▶│    │ down,
 g │ R5 ✓ ─────────────────────────────────────────▶│    │ out, and
 r │ R6   ─────────────────────────────────────────▶│  ──┘ backward
   └───────────────────────────────────────────────┘
                                                        confirmation (✓) at each step
```

Key elements as labelled in the sketch:

- **`U` blocks** — individual *units*. Atomic deliverables. Each is produced inside a
  sprint.
- **Sprint columns** (`Sprint 1`, `S2`, `S3`, … `Current sprint`) — units are grouped by
  the sprint that created them. The set of units only grows left-to-right over time.
- **`Integration`** (the vertical axis label) — the dimension along which units are
  *composed*. Going "down" means combining/depending on more of the system.
- **The grid / array** — the *regression array*. Rows are regression rounds (`R1 … R6`);
  the array spans the accumulated units.
- **`R1✓, R2✓, … R5✓, R6`** — successive regression rounds. The ✓ is a **confirmation
  gate** recorded *at each step*. `R6` is the live frontier (no ✓ yet — it's running now).
- **Arrows "down and out"** — the regression wavefront expands: *out* across the breadth
  of units in the current sprint, *down* through the depth of integration/history.
- **"backwards" + loop arrow back to `Current sprint`** — the result of the array feeds
  *back* into the current sprint (R6 → current sprint), and re-confirms *earlier* units
  when something they depend on changes.
- **"in real time / history"** — the array is recorded continuously: a live view plus an
  append-only historical record.

## 2. The three axes (what makes it an "array")

The schema is fundamentally a **3-axis lattice**, which is why complexity grows
exponentially and why it needs taming:

1. **Out (breadth / X):** units within a sprint. New work added each sprint.
2. **Down (depth / Y = integration):** how deeply a unit is composed with others —
   from "unit in isolation" → "integrated with direct deps" → "integrated with the full
   transitive closure / end-to-end".
3. **Through time (Z = regression rounds R1…Rn):** each round re-confirms the lattice
   against the current code.

A naive reading says cell-count ≈ `units × integration-depth × rounds`, and integration
depth itself trends toward all-pairs/all-subsets of units → **combinatorial explosion**.
The drawing's own arrows hint at the tame: the wave only travels where work *changed*,
and every cell carries a ✓ that can be *reused* rather than recomputed.

## 3. The motion: "down and out, backwards, with a confirmation at each step"

Read as a wavefront with four behaviours, each of which has a precise engineering meaning
(developed in `ARCHITECTURE.md`):

| Drawing phrase | Meaning | Mechanism |
|---|---|---|
| **out** | test the new breadth of the current sprint | run new units' own suites |
| **down** | integrate downward through dependencies | compose along the integration DAG |
| **backwards** | re-confirm ancestors affected by a change | reverse-dependency (impact) closure |
| **confirmation at each step** | every cell is a deterministic pass/fail gate, recorded | content-addressed result ledger (✓) |
| **loop back to current sprint** | the frontier result `R6` informs the next sprint | green root hash gates sprint exit |
| **real time / history** | live state + permanent record | append-only, hash-chained ledger |

## 4. The core problem statement (in the user's words, made precise)

> "Handle the exponential growth of complexity of the regression tests, as
> deterministically and code-based (and perhaps 'provable') as possible."

Decomposed:

- **Exponential growth** → must *not* re-run the whole lattice each round. Cost should
  scale with the **changed frontier**, not total accumulated units. (Memoized,
  impact-scoped regression.)
- **Deterministic** → identical inputs must produce identical results, byte-for-byte.
  (Hermetic execution: pinned seeds, frozen clock, no ambient network/filesystem.)
- **Code-based** → the lattice, the gates, and the routing are defined in code/data, not
  prose or manual checklists. The filesystem/graph *is* the state machine.
- **Provable** → two senses we can actually deliver:
  1. *Auditable proof of execution* — a hash-chained ledger where a single green root
     hash certifies "every confirmation in the lattice is valid for exactly this code."
  2. *Universal correctness claims* — contracts + property-based tests (∀ inputs in a
     domain), not just example-based tests; optionally model-checked invariants for
     critical units.

## 5. Mapping the drawing → the system (one-liner each)

- `U` block → **Unit**: a content-addressed module with a typed **contract** + suite.
- Sprint column → **sprint cohort** of units (provenance + version).
- `Integration` axis → **integration DAG** of declared composition edges between units.
- The grid → **regression array**: cells = (test target × integration scope), each keyed
  by a content hash.
- `✓` → **confirmation**: a cached, reusable, signed result for one cell.
- `R1…R6` → **regression rounds**: successive recomputations of the array's *changed
  frontier* only.
- "backwards" loop → **impact analysis** over the reverse-dependency closure.
- `R6 → current sprint` → **green-root gate**: a sprint can't close unless the array root
  is green.
- "real time / history" → **hash-chained result ledger** (live + append-only).

This reading is the basis for `ARCHITECTURE.md`.
