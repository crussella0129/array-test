# s25 Build Plan
1. main.rs: --version/-V/version -> "array-test <CARGO_PKG_VERSION>", exit 0; USAGE line;
   t11_cli test over all three spellings.
2. Dockerfile: builder rust:1-slim-trixie (COPY Cargo.toml/lock+src, cargo build --release
   --locked) -> debian:trixie-slim + cbmc + python3 + python3-hypothesis; COPY binary,
   T14 shim (as array-test-phase), examples; ENTRYPOINT array-test, CMD help.
   .dockerignore: target/.git/selfhost/sprints/agent-tasks/docs/tests/.github/root-md.
3. ci.yml docker job: build; --version/help smoke; quickstart round; proved-cbmc round;
   python3 -c "import hypothesis"; --privileged round + grep net_isolated in
   ledger/confirmations.ndjson; T14 shim run.
4. README "Container image" section (run modes, digest discipline, --read-only note).
5. Backlog: check C1–C3 + --version; delete HANDOFF per its instruction.
6. Verify locally (suite/clippy/fmt); PR; docker job green before merge.
