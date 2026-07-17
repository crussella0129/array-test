# Sprint s25 — Meta
- Sprint: 25
- Title: Containerization C1–C3 + --version polish
- Phase: loop
- Exit status: green (pending the PR's docker CI job — the sprint's verifier by design)
- Confidence: 0.9 locally (135 pass / 5 ignored; clippy+fmt clean); 1.0 once the docker job
  is green (no docker daemon in this session — CI is the only place the image can run)

## Done
- C1: multi-stage Dockerfile (rust:1-slim-trixie -> debian:trixie-slim; binary + cbmc +
  python3-hypothesis + T14 shim + examples; no toolchain in runtime). .dockerignore.
- C2: README "Container image" — two run modes (EnvOnly vs --privileged full sandbox),
  digest-pinning discipline, --read-only framing correction.
- C3: docker CI job — proves the shipped image: quickstart + proved-CBMC rounds green
  inside it, hypothesis imports, T14 shim runs, and under --privileged the ledger records
  net_isolated (the non-theater sandbox witness).
- Polish: --version/-V/version flag (t11_cli-covered); doubles as the image smoke command.
- HANDOFF section in agent-tasks.md read, acted on, and deleted per its own instruction;
  backlog updated (C1–C3 checked; C4/C5 + blockers + tutorial remain).

## Next
C4 (GHCR publish by digest + Distribution section), C5 (genesis parity), tutorial.
