# s2 Research Report — Testing Practice Survey & Design Review

## 1. Problem
Two inputs to this sprint: (a) a survey of established software-testing practice, looking
for anything the array-test design is missing or contradicts; (b) a critical review of the
s1 code with fresh eyes. Both feed one refactor.

## 2. Survey: the practice landscape, mapped onto this design

Each item gets a verdict: **adopt now** (this sprint's refactor), **adopt later** (backlog
task), **already core** (validates the design), or **rejected** (with reason).

### 2.1 Regression test selection / test impact analysis — *already core*
The frontier algorithm (§3) is the academic "regression test selection" problem, solved
the way modern build systems solved it: content-addressed action caching. Prior art we are
deliberately aligned with: **Bazel/Buck2** (action key = hash of inputs + command +
environment ≈ our `cell_key`), **Nix** (derivations addressed by input hashes), **Git**
(Merkle DAG as history). Microsoft's Test Impact Analysis and Google's CI literature both
report the same shape of win: selection cost must be static/structural (our DAG), not
dynamic tracing, to stay cheap. No change needed; the survey confirms the frontier design
is the industry-converged answer.

### 2.2 Merkle-tree second-preimage attacks & domain separation — **adopt now**
A Merkle construction that hashes leaves and interior nodes the *same way* admits
second-preimage confusions: an attacker (or an unlucky refactor) can present an interior
node as a leaf, or a differently-shaped tree with the same root. This is not theoretical —
Bitcoin's duplicate-transaction bug (CVE-2012-2459) and the explicit `0x00`/`0x01`
leaf/node prefixes in Certificate Transparency (RFC 6962) exist precisely because of it.

**s1 defect found:** `Hash::combine`'s doc comment claims "domain-separated combination";
the implementation is an untagged concatenation. `code_hash` and `cell_key` are both built
from the same untagged combinator, so e.g. a crafted 2-file unit and a `(path, content)`
entry hash through structurally identical byte streams. The provability story (§7.1) rests
on these keys; they must be unambiguous *by construction*, not by luck.

**Fix:** every hash gets a *derivation context* using BLAKE3's `derive_key` mode — which
exists for exactly this purpose — with frozen, versioned context strings
(`array-test/v1/code-hash`, `array-test/v1/cell-key`, `array-test/v1/file-entry`, …). Leaf
and node constructors are separate functions, so the type of hash is visible at every call
site. Context strings are **frozen forever** once a ledger commits to them; changing one is
by definition a new key universe (a full re-key event).

### 2.3 Cross-platform & filesystem determinism — **adopt now**
Determinism (§6) currently stops at the process boundary; s1's hashing does not survive the
filesystem boundary:
- `Path::to_string_lossy()` — two *distinct* non-UTF-8 filenames can lossily map to the
  same string → same hash for different content. Silent collision.
- Path separators — the same tree hashes differently on Windows (`\`) vs Unix (`/`).
- Sorting `PathBuf`s — ordering is OS-encoding-sensitive; sort the *normalized* strings.
- Symlinks — `fs::read`/`is_dir` follow them silently: a symlink cycle hangs the walk, and
  a symlink pointing outside the unit imports foreign bytes into a hash that claims to
  address only the unit. That is a hermeticity hole (§6 "no ambient I/O").

**Fix:** normalize relative paths to `/`-joined UTF-8 (reject non-UTF-8 loudly), sort by
the normalized string, reject symlinks inside `src/`.

### 2.4 Flaky-test research — *already core, one later addition*
The empirical literature (Luo et al.'s FSE'14 taxonomy; Google's public numbers on flake
rates) says flakiness is dominated by async waits, concurrency, test-order dependence, and
ambient time/network — precisely the inputs §6 pins or bans, and the determinism
meta-check + quarantine already covers detection. **Adopt later (T3/T4):** quarantine must
be *visible state* in the ledger, not a silent skip — a quarantined cell is a red mark
with a reason, otherwise quarantine becomes a place where failures go to be forgotten.

### 2.5 Mutation testing — **adopt later (new T12)**
The array certifies "all tests pass," which is only as strong as the tests. Mutation
testing (PIT for JVM, `cargo-mutants` for Rust) measures that strength: mutate the code,
require the suite to notice. Normally too expensive to run globally — but our
content-addressing changes the economics: **mutation only needs to run on the changed
frontier** (mutate only dirty units, reuse mutation scores for unchanged `code_hash`es),
the exact same memoization that tames regression cost. A per-unit mutation score becomes
guarantee-level metadata on the confirmation. This is the one survey finding that turns
into a genuinely novel synthesis with the array; recorded as backlog T12.

### 2.6 Fuzzing — **adopt later (new T13)**
Coverage-guided fuzzing (libFuzzer/`cargo-fuzz`, AFL) complements Hypothesis-style
property testing: properties check ∀-claims over *structured* generators; fuzzers find
the inputs nobody thought to generate. Fits the architecture cleanly: a fuzz corpus is a
content-addressed fixture set (`fixtures_hash`), so corpus growth re-keys cells naturally.
Optional tier, per-unit opt-in, backlog T13.

### 2.7 Metamorphic testing — *documented convention, no engine change*
When there is no oracle (common for agent-generated code: the spec is prose), metamorphic
relations substitute: you may not know `tokenize(x)` exactly, but you know
`tokenize(a ++ b)` relates to `tokenize(a)`/`tokenize(b)`. This is already expressible in
`contract.toml [properties]` — the research just names the discipline. Added to the
contract authoring guidance; also noted as input to the Phase-J judge prompt design (R-e):
a judge should *ask* for metamorphic relations when the contract lacks an oracle.

### 2.8 Snapshot/golden testing — *already core, one policy note*
`evidence_hash` over TAP output *is* golden testing. The known failure mode is golden
rot: auto-accepting new snapshots until the golden means nothing. Our design already
blocks the trap door (a changed evidence expectation changes `test_def_hash`/fixtures →
new cell, new confirmation), but policy must say: **golden updates route through Phase J**
— a judge reviews "the expected output changed" as a semantic event, never a rubber stamp.

### 2.9 Coverage — *rejected as a gate; later as metadata*
Coverage as a gate invites Goodhart's law (tests that execute lines but assert nothing).
As *metadata* it is honest and cheap: a coverage summary hash can ride along in evidence
(future T3 flag). Never a gate; the gates are Phase D + Phase J.

### 2.10 Test-size ladders — *already core*
Google's Small/Medium/Large taxonomy (strict resource envelopes per size) is our scope
ladder UNIT → DIRECT → CLOSURE → E2E with different words. The refinement worth stealing:
each scope should carry an explicit **resource envelope** (wall-clock/memory caps) enforced
by the runner — a cheap early warning that a "unit" test has quietly become an integration
test. Folded into T3's requirements.

## 3. Code review findings (s1, reviewed fresh)

| # | Finding | Severity | Action |
|---|---|---|---|
| F1 | `Hash::combine` doc claims domain separation; code has none | high (integrity) | §2.2 fix, this sprint |
| F2 | `to_string_lossy` can collide distinct non-UTF-8 paths | high (silent collision) | §2.3 fix, this sprint |
| F3 | Platform-dependent separators/sort order in `code_hash` | medium (cross-platform determinism) | §2.3 fix, this sprint |
| F4 | Symlinks silently followed (cycle hang; hermeticity escape) | medium | §2.3 fix, this sprint |
| F5 | Manifest accepts self-deps, duplicate deps, empty ids | low (caught later, but late errors are worse errors) | validate at load, this sprint |
| F6 | `Dag` lacks a topological-order accessor, though §3 step 4 requires one | low (missing forward API) | add `topo_order()`, this sprint |
| F7 | Dependencies pinned a major version behind (petgraph 0.6/0.8, thiserror 1/2) | low | bump while there are zero downstream users |

## 4. Re-key safety
The §2.2/§2.3 fixes change every hash value the system produces. This is safe **now and
only now**: no ledger exists yet (T4 is unbuilt), so no committed root refers to the old
values. After T4 ships, a change like this becomes a formal re-key event (new context
version, full array re-confirmation). Recording this as the explicit precedent: **hash
semantics changes ride ahead of the first ledger commit, or they pay for a full re-key.**

## 5. Recommendation
One refactor sprint, code-first: F1–F7 now; T12 (frontier-scoped mutation testing) and
T13 (fuzz tier) to the backlog; §2.4 quarantine visibility and §2.10 resource envelopes
folded into T3's spec; §2.7 metamorphic guidance and §2.8 golden policy into the docs.
