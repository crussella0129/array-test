# s23 Research — T8b, a live proof for the `proved` tier

The proved tier existed only as a recorded declaration (Guarantee::Proved) and an ignored
schema — no committed unit verified anything over its whole input space. The T7 property
tier's shape is the template: a real tool invoked as a TAP-emitting command, #[ignore]d with
a self-skip, provisioned in the privileged CI job.

## Provisioning the prover
Kani (the plan's named tool) installs its driver from crates.io (fine) but `cargo kani setup`
downloads a release bundle from the model-checking/kani GitHub repo. This session's egress
allows only crussella0129/array-test on GitHub — verified by probing: the scoped repo → 200,
api.github.com/repos/model-checking/kani → 403, the release-download path → 403. That is the
session's repo-scope policy, so it must not be circumvented (the reachable raw/objects CDNs
were not used to fetch the blocked repo's artifacts).

CBMC — the C bounded model checker Kani wraps — is in the Ubuntu universe archive (authorized
egress). Installing it and running a nondeterministic-input harness gives a genuine symbolic
proof: `cbmc prove.c` returned VERIFICATION SUCCESSFUL over all 256 bytes, and a deliberately
buggy variant returned VERIFICATION FAILED. So the proved tier can be made honestly live
through an allowed channel, with CBMC a faithful stand-in for (indeed, the engine of) Kani.

## Determinism
The run-twice meta-check compares evidence hashes. CBMC's raw output carries timing/paths, so
the wrapper discards it and prints fixed TAP — the cell's evidence is byte-identical across
runs, so a real proof is a clean Pass, a refuted one a clean Fail (not a quarantine).

## Honesty doctrine
A proved claim that never actually runs a prover would be exactly the "not-run reads as
passed" failure D27 forbids. So: it runs live in CI (cbmc via apt), self-skips elsewhere
(reads ignored), and a falsification test guarantees the proof is not vacuous.
