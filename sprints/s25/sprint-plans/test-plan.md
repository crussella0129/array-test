# s25 Test Plan
- AC (local): --version, -V, and version print "array-test 1.0.0" and exit 0 (t11_cli).
- AC (docker CI job — the sprint's verifier):
  1. image builds;
  2. quickstart round green inside the image (unprivileged);
  3. proved-CBMC round green inside the image (cbmc functions);
  4. `import hypothesis` succeeds (property tooling present);
  5. under --privileged, ledger/confirmations.ndjson contains "net_isolated" — the sandbox
     demonstrably applied inside the image (non-theater witness, enabled by D16 recording);
  6. array-test-phase (T14 shim) runs green inside the image.
- Gate: 135 pass / 5 ignored locally; clippy -D warnings + fmt clean; all CI jobs
  (test, privileged-tests, docker) green before merge.
