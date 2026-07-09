# s1 Research Report

## 1. Input
User pointed at `github.com/crussella0129/riteway` mid-loop, after s0 closed, asking
whether it could help the array-test design.

## 2. Findings

**riteway** is an AI-native testing framework (yours) built around RITE (Readable,
Isolated, Thorough, Explicit) and the "5 questions every unit test must answer" (unit
under test, expected behavior, actual output, expected output, reproduction). Key
elements:

- `describe`/`assert` API using a `given / should / actual / expected` shape.
- **TAP** (Test Anything Protocol) output — standardized, broadly tool-compatible.
- `Try(fn, ...args)` for exception-path testing.
- `render(jsx)` for component testing via Cheerio.
- **`riteway ai`** — a distinct mode for testing AI agents/prompts: SudoLang `.sudo` spec
  files, run N times against an agent, scored by an independent **judge agent** against a
  pass-rate **threshold** (`--runs`, `--threshold`).

## 3. Analysis — two direct hooks into the array-test design

1. **Evidence format.** `ARCHITECTURE.md` (s0 draft) invented a bespoke `evidence_hash`
   payload. riteway's `given/should/actual/expected` shape already forces every test to
   answer the same questions §7 (Provability) needs answered, and TAP is a ready-made,
   standardized wire format for that evidence — no need to invent one. → **D6**.

2. **A confirmation dimension the s0 design was missing.** Phase-D-style hermetic tests
   prove code doesn't crash and satisfies the assertions someone wrote. They cannot prove
   the code matches *intent*. `riteway ai`'s judge+threshold model is built exactly for
   that gap, but is inherently statistical (LLM judge), not hermetic-deterministic — it
   cannot be folded into the existing Merkle-root cache without corrupting the
   reproducibility claim in §7.1.

## 4. Design decision (made with the user, mid-research)
Rather than recording the judge tier as an independent, non-gating annotation, the user
specified a **two-phase gate in series**: `confirmed = det_status PASS AND judge.rating >=
threshold`. A judge failure does not fail the whole sprint — it spawns a **repair
micro-loop** (Plan→Build→Test scoped to the single rejected unit), escalating to a
sprint-level `failure-report.md` only if it exhausts a retry budget. This is now
`ARCHITECTURE.md §4` (design), §7.3 (audited-not-rooted provability boundary), §8
(on-disk `judgments.ndjson`), §9 (sprint loop integration), §10 (build order: T6, T9,
T10). → **D7**.

## 5. Risks / open questions carried forward
- **R-d (from s0, now leaning resolved):** riteway is JS/Node — implementing the array-test
  engine in Node/TypeScript is now the default assumption unless a reason emerges to
  diverge. Not yet formally locked; revisit if T1/T2 surface a blocker.
- **R-e (new):** judge model choice, prompt design, and retry-budget defaults for the
  repair micro-loop (§4.3) are unspecified — needed before T9/T10 can be built. Deferred to
  the sprint that builds T9/T10 (s2 or later), since T1/T2 (this sprint) don't depend on it.
- **R-f (new):** `judge_hash` must pin judge model *and* prompt version so a later re-audit
  of `judgments.ndjson` can tell whether a verdict used the current judge or a retired one.

## 6. Recommendation
Proceed to build T1 (content addressing + schemas) and T2 (DAG resolver) this sprint —
neither depends on the riteway/judge decisions above, so there's no reason to block on
R-e/R-f. Evidence adapter (T6) and judge gate (T9/T10) follow once the substrate exists.
