#!/bin/sh
# selfhost.verify: an intact inner state passes the full audit; a single flipped ledger
# byte fails it. Requires GNU sed (Linux userland — see selfhost/README.md).
set -e

ws="${TMPDIR:-/tmp}/array-selfhost-verify-$$"
trap 'rm -rf "$ws"' EXIT
mkdir -p "$ws/units/inner/src"

printf 'inner unit v1' > "$ws/units/inner/src/main.txt"
printf '[io]\n' > "$ws/units/inner/contract.toml"
cat > "$ws/units/inner/manifest.toml" <<'EOF'
id = "inner"
version = "0.0.1"

[test]
command = ["/bin/sh", "-c", "printf inner-ok"]
EOF

array-test run --units "$ws/units" --state "$ws/state" > /dev/null

array-test verify --state "$ws/state" > "$ws/v1"
grep -q 'VERIFIED' "$ws/v1" || { printf 'not ok 1 - intact state did not verify\n'; exit 1; }

sed -i 's/"pass"/"fail"/' "$ws/state/ledger/confirmations.ndjson"
if array-test verify --state "$ws/state" > /dev/null 2>&1; then
    printf 'not ok 1 - tampered ledger verified\n'
    exit 1
fi

printf 'ok 1 - verify accepts intact state and rejects tampering\n'
