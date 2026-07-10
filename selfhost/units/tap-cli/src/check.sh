#!/bin/sh
# selfhost.tap: the tap adapter normalizes noisy libtest output into exact, sorted,
# timing-free TAP (D14).
set -e

out=$(array-test tap -- /bin/sh -c 'printf "running 2 tests\ntest zeta ... ok\ntest alpha ... ok\ntest result: ok. finished in 9.99s\n"')

expected='TAP version 13
1..2
ok 1 - alpha
ok 2 - zeta'

[ "$out" = "$expected" ] || { printf 'not ok 1 - tap output mismatch\n'; exit 1; }
printf 'ok 1 - tap normalizes libtest output deterministically\n'
