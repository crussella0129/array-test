# s15 Test Plan — Finalized - DO NOT EDIT
- Existing AC1–AC92 stay green through the byte-preserving refactors (rot guard t15b is
  the freeze witness).
- fmt clean; clippy -D warnings clean with the new curated [lints] denies enforced.
No new ACs (hygiene sprint; behavior preserved).
