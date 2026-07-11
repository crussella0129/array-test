# s20 Test Plan

- AC: a manifest whose `id` contains `/`, `\`, `..`, a leading `.`, or a control char is
  rejected at load; a dotted id (`u.parser.tokenize`) still loads.
- AC: `safe_state_path` resolves an engine-shaped ref under the state dir and rejects
  `../escape`, `a/../../etc/passwd`, `/etc/passwd`, `./x/../..`.
- Regression: the existing judge repair-loop acceptance test still converges (the guard is
  transparent to engine-generated refs).
- Whole suite green + clippy/fmt clean.
