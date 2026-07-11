#!/bin/sh
# The `proved`-tier test command: run CBMC over the harness and translate its verdict to
# deterministic TAP. CBMC checks the assertions over the whole input space (bounded model
# checking) — a proof, not a sample — and its verdict is deterministic, so the cell passes
# array-test's run-twice determinism meta-check.
#
# Only the fixed TAP lines go to stdout (the hash-committed evidence); CBMC's own output
# (which carries timing/paths that would vary between runs) is discarded, keeping the
# evidence byte-identical across the meta-check.
set -u

# cwd is the unit dir (the hermetic runner sets it), so the harness is at src/prove.c.
if cbmc src/prove.c >/dev/null 2>&1; then
    printf 'TAP version 13\n1..2\nok 1 - nibble split/recombine is the identity for all u8\nok 2 - every nibble encodes to a valid hex digit\n'
    exit 0
else
    printf 'TAP version 13\n1..2\nnot ok 1 - cbmc verification failed\n'
    exit 1
fi
