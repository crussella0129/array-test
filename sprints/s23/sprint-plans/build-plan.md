# s23 Build Plan
1. examples/proved-cbmc/units/nibble-roundtrip/: prove.c (nondet byte, 2 assertions),
   run-proof.sh (cbmc -> deterministic TAP), contract.toml, manifest.toml (guarantee =
   "proved", explicit PATH). README for the example.
2. tests/t8b_proved.rs: non-ignored proved-plumbing test; #[ignore]+self-skip real-proof
   test (pass + Guarantee::Proved); #[ignore]+self-skip falsification test (red round).
3. CI: privileged-tests installs cbmc; runs --ignored.
4. Validate live here (cbmc installed): run the unit via the CLI and the tests via --ignored.
5. Verify: full suite, clippy -D warnings, fmt --check.
