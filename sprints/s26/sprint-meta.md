# Sprint s26 — Meta
- Sprint: 26
- Title: C4 publish-by-digest + C5 genesis parity + Kani handoff
- Phase: loop
- Exit status: green (publish job verified on the post-merge main push — main-only by design)
- Confidence: 1.0 for docs + docker verify step once PR CI green; publish confirmed on main

## Done
- C4: `publish` CI job — GHCR on green main pushes only (needs test+privileged+docker),
  GITHUB_TOKEN, crate-version + sha tags, digest in the job summary as the consumable pin.
  README "Distribution" section (image by digest / cargo install / library).
- C5: TEMPLATE.md container-path genesis (builder image runs the ritual — selfhost cells
  need cargo; --user/CARGO_HOME ownership notes); docker CI job now re-verifies the
  committed founding ledger with the runtime image (zero-toolchain audit) on every push.
- Backlog: C1–C5 closed; array-test-fork marked user-owned; Kani provisioning handoff
  (delete-after-reading) parked at the bottom of agent-tasks.md.

## Honest split (the C5 finding)
Genesis *creation* needs the toolchain (relative target/debug PATH in selfhost cells);
genesis *verification* doesn't. Builder image for the former, runtime image for the
latter — documented as such rather than overclaiming. See D37.

## Known human steps
- Make the GHCR package public if desired (first push creates it private).
- The sprint-loops fork: user-owned, no agent action.

## Next
Kani handoff (authorized session), authoring tutorial, cross-platform decision.
