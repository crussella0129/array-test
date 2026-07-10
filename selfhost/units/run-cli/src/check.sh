#!/bin/sh
# selfhost.run: a full inner round over a generated workspace — green, cached on the
# second round, byte-identical root. The inner root is machine-independent because
# cell keys contain no paths and the fixture content, seed, and toolchain sentinel are
# all fixed (s10 research §2.1).
set -e

ws="${TMPDIR:-/tmp}/array-selfhost-run-$$"
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

array-test run --units "$ws/units" --state "$ws/state" > "$ws/out1"
grep -q 'ALL PASS' "$ws/out1" || { printf 'not ok 1 - first round not green\n'; exit 1; }

array-test run --units "$ws/units" --state "$ws/state" > "$ws/out2"
grep -q '1 reused' "$ws/out2" || { printf 'not ok 1 - second round did not reuse\n'; exit 1; }

root1=$(grep -o 'root blake3:[0-9a-f]*' "$ws/out1")
root2=$(grep -o 'root blake3:[0-9a-f]*' "$ws/out2")
[ -n "$root1" ] && [ "$root1" = "$root2" ] || { printf 'not ok 1 - roots differ across cached rounds\n'; exit 1; }

printf 'ok 1 - run executes, caches, and reproduces the root\n'
