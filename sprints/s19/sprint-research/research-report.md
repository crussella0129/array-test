# s19 Research — F3, over-long functions

Measured function line-spans across `src/`. The four longest: full_audit (153),
run_mutation (139), run_round (131), run_cell (128). The rest sit under ~100.

"A function should do one thing" (Martin) is the lens, but the operative test here is
*independent testability*: can a reader verify one concern without simulating the others?
- full_audit interleaves four independent verifications (confirmations, roots, sidecar
  chains, evidence) sharing one report — the textbook case for phase extraction. Each
  phase becomes separately readable and, if wanted later, separately unit-testable.
- run_round mixes tier-gating control flow with the per-cell frontier decision. The cell
  decision (gated-skip vs cache-hit vs run-store-cache, plus quarantine) is a closed unit
  worth its own function; the loop is then purely about gating.
- run_mutation is a doubly-nested loop (units × mutants) with shared invariants. The clean
  decomposition needs a home for those invariants; passing them as args trips
  too_many_arguments, so a borrowing context struct is the idiomatic Rust answer.

run_cell is different in kind: it is a syscall/namespace sandbox whose privileged branches
only execute under the privileged CI job (#[ignore]d locally). Decomposing unsafe fork/exec
setup carries more risk than the length alone justifies; deferred to a focused pass so the
low-risk wins ship cleanly first.

No behavior may change: these functions sit under the frozen hashing and ledger layers, so
the sprint's success criterion is "every existing test still passes, byte-for-byte" — the
rot guard and the audit/mutation acceptance tests are the proof.
