# s22 Test Plan
- No behavior change ⇒ no new tests; proved by the existing suite staying green.
- Witnesses: t3b mem-cap (non-ignored, exercises install_sandbox setrlimit branch in the
  normal job); t3b netns + t3c mount (ignored, exercise the unshare/mount branches in the
  privileged job); t3/t3b general runner tests for build_cell_command + wait_with_envelope.
- Gate: 130 pass / 3 ignored locally; all four CI checks (test + privileged-tests, both runs)
  green before merge.
