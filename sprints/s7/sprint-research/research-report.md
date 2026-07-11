# s7 Research Report — The Guarantee Tiers Become Code

## 1. Direction
User-directed (after s6): direction 3 — the guarantee tiers. T7 (property), T8 (formal),
T9 (Phase-J judge gate), T10 (repair micro-loop). T14 remains parked.

## 2. Design decisions

### 2.1 Guarantee levels are declarations, audited — not verified — by the engine (T7/T8)
A cell declares `guarantee = "example" | "property" | "proved"` in its TestSpec
(default `example`). The engine cannot verify that a command really ran Hypothesis or
Kani — what it CAN do is (a) hash the declaration into `test_def_hash` so changing the
claimed level re-keys the cell, (b) record the level in the chained ledger per §7.2, and
(c) hand Phase J the declaration to audit ("this claims to be a property test — is it?").
Honest division of labor: declarations in the deterministic layer, judgment of
declarations in the judged layer.

### 2.2 T7 property tier: Hypothesis over the existing boundary
Nothing new in the engine — that's the point of D8/D14's boundaries. A property cell is
`python3 <script>` where the script runs Hypothesis with
`derandomize=True`/pinned seed and emits TAP. This environment has Python 3.11.15 +
Hypothesis 6.156.4; the acceptance test runs a real Hypothesis property (with a
graceful skip if a host lacks the interpreter — capability-gated like AC53).

### 2.3 T8 formal tier: schema now, live Kani deferred (T8b)
`guarantee = "proved"` is accepted and recorded now; a proving cell is just another
TAP-emitting command. Actually *running* Kani requires a multi-GB toolchain install —
environment-gated, deferred to **T8b** with the same skip-pattern as 2.2 when it lands.
Scoping this honestly: s7 ships the schema and the audit trail, not a proof.

### 2.4 T9 judge gate: the judge is a command (→ D17)
Everything in this system is a command emitting parseable output — the judge too. A
workspace opts in with `<units-dir>/judge.toml`:

```toml
command    = [...]     # receives ARRAY_TEST_UNIT_DIR/_UNIT_ID/_SCOPE/_EVIDENCE/_CONTRACT
runs       = 3         # independent judge passes (riteway ai's N-runs model)
threshold  = 100       # percent of runs that must clear min_rating
min_rating = 80        # a run passes if its rating >= this

[repair]               # optional: enables the T10 micro-loop
command = [...]        # receives ARRAY_TEST_UNIT_DIR + ARRAY_TEST_CRITIQUE
budget  = 2
```

Protocol: the judge writes its critique to stdout; the **last line** must be
`rating: <0-100>`. The judge runs *unhermetically* (an LLM judge needs network) — which
is exactly why D7 keeps Phase J out of the deterministic root. Its identity is pinned:
`judge_hash` = hash of command+config, so a changed judge prompt is a new judge.

**Judgments get the same economics as confirmations:** verdicts are cached keyed by
`(cell_key, judge_hash)` — an unchanged cell judged by an unchanged judge is not
re-judged — and recorded in their own hash-chained `judgments.ndjson` with critique
transcripts under `ledger/critiques/` (§7.3/§8, now real).

Phase J runs only over a det-green round (§4.2), and only det-Pass cells are judged.

### 2.5 Evidence store (audit gap found during design)
Judges need to *read* evidence, which exposed a gap: we committed evidence hashes but
discarded the bytes. Fix: a content-addressed evidence store
(`state/evidence/<evidence_hash>.tap`) written for every executed cell. This
independently strengthens §7.1 — a root is now backed by retrievable, hash-verified
evidence, not just hashes of discarded data.

### 2.6 T10 repair micro-loop: rounds ARE the loop (→ D18)
§4.3 lands with almost no new machinery: on judge rejection, the repair command edits
the unit; the next attempt is simply **another det round** — the changed unit re-keys,
the frontier machinery re-runs exactly what changed, and Phase J re-judges (cache
misses only where the key moved). Attempts are ordinary numbered rounds in history.
Budget exhausted → a consumer-agnostic failure record
(`ledger/failures/R<k>-judgment.md`, critique refs included) — T14's shim can translate
it to sprint-loops' `failure-report.md` later. Escalation stays visible either way.

## 3. Recommendation
Build: guarantee levels (manifest/ledger/test_def), evidence store, judge.rs (config,
protocol, judgment ledger, cache, gate), repair loop, CLI wiring (`run` uses the judged
path when judge.toml exists; exit 0 iff det-green AND judge-green). Tests with
scripted sh judges/repairers (deterministic, no LLM dependency in CI) + a real
Hypothesis property cell.
