# s26 Build Plan
1. ci.yml: docker job += runtime-image `verify --state` over a read-only mount of
   selfhost/state (C5 witness). New `publish` job: main-push only, needs all three jobs,
   packages:write, GHCR login with GITHUB_TOKEN, version+sha tags, digest to job summary.
2. TEMPLATE.md: container-path genesis ritual (builder image, --user/CARGO_HOME notes,
   runtime-image verify line).
3. README: "Container image" closing paragraph -> "Distribution" section (image by digest /
   cargo install / library).
4. agent-tasks.md: close C4/C5; mark array-test-fork user-owned; append the Kani
   delete-after-reading handoff.
5. D37 + sprints/s26 records. PR; merge on green; then confirm the publish job on the
   main push (the only place it can run) and report the digest.
