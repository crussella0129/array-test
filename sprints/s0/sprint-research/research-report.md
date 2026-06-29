# s0 Research Report

## 1. Problem
Build an agentic testing system matching the hand-drawn schema: units (`U`) produced per
sprint, an integration/regression array that travels "down and out, backwards, with a
confirmation at each step," looping back into the current sprint and recorded in real time
+ history. It must tame the exponential growth of regression complexity while being
deterministic, code-based, and "provable."

## 2. Source artifacts
- The schema sketch (analyzed in `docs/SCHEMA-ANALYSIS.md`).
- The sprint-loops canonical spec (Research→Plan→Build→Test→Loop; filesystem-as-state).

## 3. Findings
- The "array" is best modelled as a **Merkle DAG of confirmations** (`ARCHITECTURE.md`).
- Exponential growth has three identifiable sources (all-subsets integration, full-history
  re-runs, broad change blast-radius) and each has a concrete lever (DAG-scoped
  integration, content-addressed memoization, reverse-dep impact closure).
- "Provable" is deliverable in two honest tiers: an always-on auditable execution root,
  plus scaled property/contract/formal guarantees recorded per cell.
- Determinism (hermetic execution) is the *precondition* — without stable keys the cache,
  and therefore the cost model, collapse.

## 4. Risks / open questions
- **R-a:** Choice of hash + content-canonicalization must be stable across platforms.
- **R-b:** Property-based generators must themselves be seed-deterministic.
- **R-c:** Defining the `E2E` scope's "entrypoints" needs a convention (manifest flag?).
- **R-d:** Implementation language/toolchain not yet chosen — affects hashing & hermeticity
  (candidate: Rust or Python; decide in s1 with determinism as the deciding criterion).

## 5. Recommendation
Proceed to a design sprint exit (s0), then s1 builds the substrate (T1 content
addressing/schemas, T2 DAG resolver) since every later cell key depends on them.
