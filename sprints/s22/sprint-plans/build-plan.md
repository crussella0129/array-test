# s22 Build Plan
1. runner.rs: extract build_cell_command(spec, program) -> Command (env + hygiene + seed +
   piped stdio + unix install_sandbox).
2. Extract install_sandbox(cmd, spec) (unix): process_group + pre_exec closure, verbatim,
   plus a # Safety doc.
3. Extract wait_with_envelope(child, timeout, start) -> (Option<ExitStatus>, bool).
4. run_cell calls the three; imports Child/ExitStatus.
5. Verify: full suite (t3b mem-cap non-ignored), clippy -D warnings, fmt --check. Privileged
   CI job validates the netns/mount branches.
