# s26 Research — publishing and genesis parity

## C4 — what "publish by digest" requires
GHCR accepts pushes authenticated with the workflow's GITHUB_TOKEN given
`permissions: packages: write` — no PAT, no secret to provision. Gating: publish must never
ship an un-proven image, so `needs: [test, privileged-tests, docker]` and main-push only.
Tags (crate version, sha-<commit>) are human conveniences; the registry digest is the pin,
so the job prints a ready `docker pull …@sha256:…` into the step summary. Limitation: a
main-only job cannot be exercised by PR CI — it is verified on the merge commit's run.
First-push visibility: GHCR packages start private; making them public is a human Settings
step (the token cannot).

## C5 — where parity actually lies
Read the selfhost units: every cell resolves the CLI via a relative
`../../../target/debug` PATH entry — the ritual builds the binary it then certifies, so
genesis CREATION needs cargo/rustc. The runtime image deliberately carries no toolchain
(C1). Two dishonest options were rejected: putting the toolchain in the runtime image
(destroys the lean-runtime point), or claiming the runtime image can run the ritual (it
cannot). The honest split: run the ritual in the *builder* image — the identical pinned
environment the published binary compiles in — and let the *runtime* image do what it can
do fully: independently re-verify a founding ledger with zero trust and zero toolchain.
The docker CI job proving the latter against this repo's own genesis makes the claim live
on every push rather than documentary.

## Kani handoff
The plan parked in agent-tasks.md encodes the landmines already paid for in s23/s25 so a
future session does not rediscover them: the CARGO_TARGET_DIR-inside-src re-key trap
(code_hash walks src/**), the cleared-env declarations cargo kani needs (PATH/HOME/...),
the determinism wrapper pattern, offline-bundle provisioning as the egress fallback, CI
caching of ~/.kani, and log-verified (not merely green) execution as the bar (s23
precedent).
