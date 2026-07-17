# s25 Research — containerizing the evidence-producing environment

## Why an image, here specifically
Three project-specific fits (from the usability assessment + the docker discussion):
1. Provisioning: proved/property need cbmc/hypothesis; baking them in turns "provision it
   yourself" into "live by default".
2. Linux guarantee: the sandbox is Linux-only; an image makes that explicit, and the CI
   privileged-tests job already proved the sandbox works INSIDE a container (netns + mount
   tests pass under --privileged --cap-add=SYS_ADMIN) — so sandbox-in-container is
   verified, not speculative.
3. Doctrine fit: digest-pinned image = content addressing applied to the runner's
   environment, the counterpart of toolchain_hash for the tested toolchain.

## Key facts established before building
- rust:latest (Debian trixie family) apt-installs cbmc 6.6 — seen live in the s23 CI logs;
  so debian:trixie-slim has cbmc + python3-hypothesis in apt. Builder and runtime pinned to
  the same Debian release for glibc compatibility.
- IsolationLevel serializes snake_case: the ledger literally contains "net_isolated" or
  "env_only" per confirmation (D16). That makes a *behavioral* CI assertion possible: grep
  the ledger inside the privileged container — green rounds alone cannot distinguish
  sandboxed from EnvOnly, the recorded level can.
- `help` exits 0; `--version` did not exist (exit 2) — fixed this sprint since the smoke
  test wants a stable zero-exit no-op.
- Full `cargo test --include-ignored` inside docker build is not feasible (RUN steps can't
  get --privileged on stock GitHub runners), and the runtime image deliberately has no
  toolchain. So C3 proves the *shipped artifact* via `docker run` steps instead — which is
  the more honest claim anyway (users run the image, not the build).

## Framing correction (from the user discussion, kept)
"Ultimate immutability" = image layers, yes; running container, no. Document --read-only
(+ --tmpfs for state) for runtime immutability. What the image really buys is environment
reproducibility — exactly the kind this tool trades in.

## Session constraint
No docker daemon here: the Dockerfile is CI-verified (the established green-before-merge
rule makes this safe), with paths/tools cross-checked locally beforehand.
