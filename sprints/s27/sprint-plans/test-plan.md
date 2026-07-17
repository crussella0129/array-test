# s27 Test Plan
- AC: after change then revert, the root equals the pre-change root, 0 cells executed,
  all reused; ledger audits clean and retains every round (append-only, HEAD tracks tree).
- AC: a fresh/empty state dir over reverted content reproduces the same root by
  re-execution (rollback floor — no cache dependency).
- Gate: 137 pass / 5 ignored; clippy -D warnings + fmt clean; frozen surfaces untouched.
