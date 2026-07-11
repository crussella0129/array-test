# s24 Research — T14, wiring array-test into sprint-loops' Test phase

array-test exists to be the Test phase of sprint-loops (Research -> Plan -> Build -> Test ->
Loop; the filesystem is the state machine). D11 forbids the core from knowing about
sprint-loops, so T14 must be an adapter that depends on array-test, never the reverse.

The shape is already implied by array-test's own design, which this session has been living
inside: a sprint accretes units; the Test phase is a round; a green root is the pass; the
persistent ledger/roots/evidence is the durable record; frontier reuse makes each sprint's
Test phase re-run only what changed. So the adapter is small on purpose — a script that maps
a sprint-loops project layout onto `array-test run` + `verify`, gates on green, and leaves a
per-sprint record. No engine changes; the agnostic core stays agnostic.

The falsification the tests must guarantee: a broken unit has to FAIL the phase (exit 1), or
the gate is theater. And a green phase must both pass AND re-verify (zero trust in the
runner) before it advances the loop.

## Provisioning note
The user asked for this on the sprint-loops side in a fork `array-test-fork`. The session's
GitHub scope is the array-test repo only — repo creation/fork for others returns 403
(confirmed). Per the same discipline as D34, this is reported, not routed around; the
adapter is built in-scope and made copy-ready for the fork.
