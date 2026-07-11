# Sprint s23 — Meta
- Sprint: 23
- Title: T8b — the proved tier made live (CBMC, Kani's engine)
- Phase: loop
- Exit status: green
- Confidence: 1.0 (131 pass / 5 ignored normally; 136 with CBMC; clippy -D warnings + fmt clean)

## Done
- Committed examples/proved-cbmc: a C harness CBMC verifies over ALL 256 bytes (hex-nibble
  round-trip + hex-digit validity — the invariant behind Hash::hex), a TAP wrapper, a
  manifest declaring guarantee = "proved", and a README.
- tests/t8b_proved.rs: non-ignored proved-plumbing test (records Guarantee::Proved
  everywhere) + two #[ignore]+self-skip tests (real proof passes & records Proved; a
  falsified harness turns the round red). Validated live this session with CBMC.
- CI: privileged-tests installs cbmc and runs it via --ignored — the proof runs in CI.

## The egress wall (recorded)
cargo kani setup downloads from the model-checking/kani GitHub repo; session egress scopes
GitHub to crussella0129/array-test only (others 403). Not routed around; used CBMC (Kani's
engine) from the Ubuntu archive instead. See D34.

## Honesty
D14/D19/D27 applied to a headline claim: proved now demonstrably verifies over the whole
input space in CI; absent the prover the tests read ignored, never falsely passed.

## Next
s24 / T14 — sprint-loops adapter (array-test-fork). Kani (Rust path) if its host is ever
authorized.
