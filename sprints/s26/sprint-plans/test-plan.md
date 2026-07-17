# s26 Test Plan
- AC (PR CI): docker job's new step — the runtime image re-verifies the committed founding
  ledger (VERIFIED, exit 0) from a read-only mount.
- AC (post-merge, main push): publish job pushes ghcr.io/crussella0129/array-test at the
  crate version + sha tags and prints the digest in the job summary.
- Docs: TEMPLATE genesis container path names the builder image and the ownership flags;
  README Distribution says digest-only consumption.
- Gate: all PR jobs green before merge; publish confirmed green on the merge commit.
- No Rust changes: local suite/clippy/fmt stay at s25 values (135 pass / 5 ignored).
