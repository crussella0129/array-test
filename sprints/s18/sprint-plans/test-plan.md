# s18 Test Plan
- AC: a manifest with `[tests.galactic]` fails to load (parse error).
- AC: manifests with `[tests.unit|direct|closure|e2e]` load successfully.
- Regression: t5b unknown-scope-rejected-at-load still green.
- Freeze: t15b_durable rot guard green (byte-for-byte ledger verification).
- Whole suite + clippy/fmt clean.
