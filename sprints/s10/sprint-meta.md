# Sprint s10 — Meta

- **Sprint:** 10
- **Title:** T15b — the durable ledger, the v1 freeze, version 1.0.0
- **Phase:** loop (research, build, test complete)
- **Started:** 2026-07-10
- **Exit status:** green
- **Confidence:** 1.0 (founding rounds green on first execution; full suite green)

## Goal
Produce the first durable committed ledger — triggering D9's freeze — via a
machine-independent self-host workspace, and declare 1.0.0.

## Definition of done
- [x] `selfhost/units`: tap/run/verify units driving the built CLI via relative PATH
  (no absolute paths anywhere); scripts in `src/` (inside `code_hash`); real dep edge
  with closure scope; `toolchain.lock` pinned.
- [x] Founding rounds executed live (AC74): R1 = 3 executed, R2 = 3 reused,
  byte-identical root `blake3:70258f45…`, `verify` → VERIFIED.
- [x] `selfhost/state` committed (ledger, R1/R2 certificates, cache, evidence store).
- [x] Rot guard `tests/t15b_durable.rs` (AC75/AC76): full audit clean over the
  committed state; R2 all-reused with identical root — pure file verification,
  machine-independent.
- [x] Freeze declared: D21; `hash.rs` domain docs marked FROZEN; version **1.0.0**.
- [ ] Committed & pushed.

## Notable
The machine-independence key was two facts composing: exec resolves *relative PATH
entries* against the cell's cwd, and cell keys contain no paths — so a committed
workspace can drive the locally built binary anywhere, and even the inner round's root
asserted by `selfhost.run` is identical across machines. Self-hosting made the
schema's loop-back arrow literal one last time: the machine that certifies work is now
permanent, tamper-evident history *inside its own array*.

## Next
Post-1.0 backlog, all extending against frozen keys per D20: T14 (user's call on which
side), T7b, T8b, T12/T13, T3c.
