# s22 Research — run_cell decomposition

D31 measured run_cell at 128 lines and deferred it: unlike the audit/round/mutation
functions, its bulk is an `unsafe` fork/exec sandbox whose isolation branches only run in
the privileged CI job. The deferral was about risk, not length — so the research question
here is "can this be split without touching behavior, and is every branch still tested?"

## Seams
run_cell is three phases: (1) construct the child Command incl. env hygiene and the unix
sandbox; (2) spawn, drain pipes on threads, wait under the wall-clock envelope; (3) classify
exit → RunStatus and hash evidence. Phases 1 and 2 are the extractions; phase 3 is already
short. The `pre_exec` closure and the SIGKILL-the-group loop move verbatim into helpers —
no logic changes, only relocation.

## Safety documentation
The pre_exec unsafe block was previously uncommented. Extracting it into install_sandbox is
the moment to add the `# Safety` contract (post-fork/pre-exec: async-signal-safe libc only,
heap-free moved captures, fail-closed), matching the trust-boundary doctrine s20 (F17/F21)
established for run_repair and make_root_readonly. This is the last under-documented unsafe.

## Coverage check (the deferral's actual concern)
- setrlimit (memory cap) needs no privilege → the non-ignored t3b mem-cap test covers it in
  the normal CI job.
- unshare(CLONE_NEWNET/NEWNS) + mount_setattr need CAP_SYS_ADMIN/userns → the #[ignore]-gated
  t3b/t3c tests cover them in the privileged CI job.
So every install_sandbox branch is exercised on the PR across the two jobs — the decomposition
is safe to land.
