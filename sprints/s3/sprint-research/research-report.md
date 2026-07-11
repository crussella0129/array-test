# s3 Research Report — Embedding Contract + Runner/Ledger Design

## 1. Input
User direction: array-test will become central to the **Test phase of the sprint-loops
protocol** (`crussella0129/sprint-loops`), but must remain **a separate thing** — usable
standalone, embeddable in any application, by anyone.

## 2. The embedding contract (→ D11)

The only architecture that satisfies both goals is **library-first with one-directional
coupling**:

- array-test **never references sprint-loops**. No sprint-loops paths, no phase names, no
  knowledge that a consumer exists. It is a Rust library (`array_test`) plus, later, a
  thin CLI (T11) over that library.
- What consumers get is a **stable output contract**, not an integration:
  1. **Exit semantics** — a round is green iff the array root is all-PASS.
  2. **`roots/R<k>.json`** — machine-readable round certificate (round, root hash, cell
     count, all-pass flag).
  3. **`confirmations.ndjson`** — the append-only, hash-chained ledger; independently
     re-verifiable by anyone with the file (no trust in the runner).
  4. **TAP evidence** — per-cell raw output, hash-committed in the ledger.
- sprint-loops' Test-phase particle then becomes a *consumer shim* (backlog **T14**, lives
  on the sprint-loops side or as an optional adapter doc here): run the round via the
  library/CLI, translate `roots/R<k>.json` into `test-report.md` / `failure-report.md`,
  map all-pass onto the phase exit condition. sprint-loops' file conventions never leak
  into array-test's core.

This is the same shape riteway/TAP already gave us for evidence (D6): a boundary defined
by stable formats, not by shared code.

## 3. T3 runner design decisions

- **Hermetic env, v1**: `env_clear()` + only declared variables + a fixed hygiene set
  (`TZ=UTC`, `LC_ALL=C`, `SOURCE_DATE_EPOCH=0`) + `ARRAY_TEST_SEED`. `PATH` is passed
  through explicitly by the caller if needed (binaries are a `toolchain_hash` concern).
- **Evidence** = framed `(stdout, stderr, exit_code)` with length prefixes, hashed under
  a new `array-test/v1/evidence` context. stderr is *included*: if diagnostics are
  nondeterministic, the determinism meta-check should say so, not look away.
- **Determinism meta-check** = run twice, compare evidence hashes; mismatch →
  **Quarantined**, a first-class, ledger-visible status (D10), never a silent skip.
- **Resource envelope, v1**: wall-clock timeout, enforced by kill; breach is a distinct
  `TimedOut` status (envelope breach ≠ test failure, per D10's early-warning rationale).
- **Known gap (R-g, recorded honestly):** v1 does **not** block network or cap memory —
  §6's "no ambient I/O" is enforced only by env hygiene + the meta-check catching
  nondeterminism it causes. Full isolation (rlimits, network namespaces/seccomp) is
  backlog **T3b**. Until then, the determinism claim for a cell is "meta-checked", not
  "sandbox-guaranteed", and the ledger records which.

## 4. T4 ledger design decisions

- **Entries** are canonical fixed-length byte encodings (no JSON canonicalization
  games), hashed under `array-test/v1/ledger-entry`, chained via `prev`; genesis sentinel
  under its own context. The ndjson line is the human/tooling view; the hash covers the
  canonical bytes.
- **Timestamps** live inside the chained entry hash (append-only history may record wall
  time — chain verification replays recorded values), but are **excluded from the array
  root**, which commits only to `{cell_key → det_status}` and therefore stays
  reproducible from a fresh re-run.
- **Array root, v1**: sorted-by-cell-key sequence of `leaf(root-leaf, cell_key ‖ status)`
  combined under `array-test/v1/array-root`. A flat ordered commitment — inclusion
  proofs (true Merkle paths) are a later, compatible upgrade since the leaf layer is
  already defined.
- **`det_status` widens** to `Pass | Fail | Quarantined | TimedOut` — the latter two are
  the D10 "visible state" requirement landing in the schema. Root is green iff **all**
  cells are `Pass`.

## 5. Freeze note
This sprint adds contexts (`evidence`, `ledger-entry`, `ledger-genesis`, `root-leaf`,
`array-root`) — additive, allowed under D9. The moment the first real ledger commits, the
whole v1 context family freezes.

## 6. Recommendation
Build T3 + T4 as library modules this sprint; record D11; add T14 (sprint-loops adapter)
and T3b (full sandbox) to the backlog. CLI (T11) next sprint so the standalone story gets
a binary, not just a crate.
