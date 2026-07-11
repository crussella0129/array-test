# `proved`-tier example — a live bounded-model-checking proof

array-test's guarantee tiers (`example | property | proved`, ARCHITECTURE.md §7.2) are a
*declaration* the engine records and Phase J audits — the engine runs whatever command the
unit provides and confirms its exit status. `proved` is the strongest tier: the unit's
command should verify its claim over the **entire input space**, not sample it.

This unit does exactly that with [CBMC](https://www.cprover.org/cbmc/) — the C bounded
model checker that [Kani](https://model-checking.github.io/kani/) wraps for Rust.

## The unit

- [`units/nibble-roundtrip/src/prove.c`](units/nibble-roundtrip/src/prove.c) — a harness
  with a *nondeterministic* byte, so CBMC checks the assertions for all 256 values at once.
  The property mirrors the invariant behind `Hash::hex` in the engine: splitting a byte
  into hex nibbles and recombining it is the identity, and every nibble maps to a valid
  lowercase hex digit.
- [`units/nibble-roundtrip/src/run-proof.sh`](units/nibble-roundtrip/src/run-proof.sh) —
  runs `cbmc` and emits deterministic TAP (CBMC's own output, which carries timing, is
  discarded so the hash-committed evidence is byte-identical across the run-twice
  determinism meta-check).
- [`units/nibble-roundtrip/manifest.toml`](units/nibble-roundtrip/manifest.toml) —
  declares `guarantee = "proved"` and the explicit `PATH` the hermetic cell needs.

## Running it

Install CBMC (`apt-get install -y cbmc` on Debian/Ubuntu), then:

```
array-test run --units examples/proved-cbmc/units --state /tmp/proved-state
```

The cell passes iff CBMC verifies the harness; the confirmation is recorded at the
`proved` level in the ledger. `tests/t8b_proved.rs` exercises this end to end (including a
falsification case that makes a refutable assertion turn the round red); it is `#[ignore]`d
so a host without CBMC reports it as *ignored*, never falsely *passed*, and the CI
`privileged-tests` job installs CBMC and runs it for real.

## Why CBMC and not Kani here

Kani is the Rust-native path and would prove Rust harnesses directly. Its distribution
bundle lives in a GitHub repo outside this project's authorized egress scope, so this
example uses CBMC — Kani's underlying engine — which installs from the OS package archive.
The `proved` tier is prover-agnostic: any command that verifies its claim and exits
non-zero on refutation is a valid proved-tier cell.
