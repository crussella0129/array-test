# s24 Test Plan
- AC: a green project passes the phase (exit 0) and writes a test-record.md with the root.
- AC: a broken unit fails the phase (exit 1) and the record says RED.
- AC: a missing binary is a usage error (exit 2), not a false pass.
- AC (manual/live): re-running an unchanged project reuses the frontier (0 executed, same
  root) — the incremental cross-sprint economics.
- Gate: 134 pass / 5 ignored; clippy -D warnings + fmt clean.
