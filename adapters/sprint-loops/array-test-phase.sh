#!/bin/sh
# array-test as the Test phase of a sprint-loops sprint (T14).
#
# sprint-loops runs Research -> Plan -> Build -> Test -> Loop, with the filesystem as the
# state machine. This script IS the Test phase: it runs one array-test round over the
# sprint's units and gates the sprint on a green root. Because array-test's state
# (ledger + roots + evidence) persists across sprints, each sprint re-runs only the cells
# whose inputs changed — the "integration/regression array travelling backwards with a
# confirmation at each step" from the founding schema, made incremental.
#
# Exit 0 iff the round is green (the Test phase passes). Any other exit fails the phase.
#
# Usage:
#   array-test-phase.sh --project <dir> [--sprint <name>] [--seed N]
#   array-test-phase.sh --units <dir> --state <dir> [--seed N]
#
# Layout convention (--project mode): <project>/units/ holds the accreting unit workspace;
# <project>/.array-test/state/ is the durable cross-sprint state; a per-sprint Test record
# is written to <project>/sprints/<sprint>/test-record.md when --sprint is given.
#
# The binary is found via $ARRAY_TEST_BIN, else `array-test` on PATH.
set -eu

BIN="${ARRAY_TEST_BIN:-array-test}"
PROJECT=""
SPRINT=""
UNITS=""
STATE=""
SEED="0"

while [ $# -gt 0 ]; do
    case "$1" in
        --project) PROJECT="$2"; shift 2 ;;
        --sprint)  SPRINT="$2";  shift 2 ;;
        --units)   UNITS="$2";   shift 2 ;;
        --state)   STATE="$2";   shift 2 ;;
        --seed)    SEED="$2";    shift 2 ;;
        -h|--help) sed -n '2,20p' "$0"; exit 0 ;;
        *) echo "array-test-phase: unknown argument '$1'" >&2; exit 2 ;;
    esac
done

if [ -n "$PROJECT" ]; then
    [ -z "$UNITS" ] && UNITS="$PROJECT/units"
    [ -z "$STATE" ] && STATE="$PROJECT/.array-test/state"
fi

if [ -z "$UNITS" ] || [ -z "$STATE" ]; then
    echo "array-test-phase: need --project, or both --units and --state" >&2
    exit 2
fi
if ! command -v "$BIN" >/dev/null 2>&1 && [ ! -x "$BIN" ]; then
    echo "array-test-phase: array-test binary not found (set ARRAY_TEST_BIN or add it to PATH)" >&2
    exit 2
fi
if [ ! -d "$UNITS" ]; then
    echo "array-test-phase: units dir '$UNITS' does not exist" >&2
    exit 2
fi
mkdir -p "$STATE"

# --- Test phase: one round, gated on green. ------------------------------------------
run_out=$("$BIN" run --units "$UNITS" --state "$STATE" --seed "$SEED" 2>&1) && phase_ok=1 || phase_ok=0
echo "$run_out"

# The round's root is the sprint's Test-phase certificate; pull it from the run summary.
root=$(printf '%s\n' "$run_out" | sed -n 's/.*root \(blake3:[0-9a-f]*\).*/\1/p' | tail -1)

# Independent re-verification of the chain + latest root (zero trust in the runner).
if [ "$phase_ok" = "1" ]; then
    "$BIN" verify --state "$STATE" >/dev/null 2>&1 || {
        echo "array-test-phase: round was green but verification failed — treating as RED" >&2
        phase_ok=0
    }
fi

# --- Optional per-sprint Test record (the sprint-loops sprint's durable evidence). ----
if [ -n "$SPRINT" ] && [ -n "$PROJECT" ]; then
    rec_dir="$PROJECT/sprints/$SPRINT"
    mkdir -p "$rec_dir"
    {
        echo "# Test phase — sprint $SPRINT"
        echo
        if [ "$phase_ok" = "1" ]; then
            echo "- Verdict: **GREEN** (Test phase passes)"
        else
            echo "- Verdict: **RED** (Test phase fails; loop back to Build)"
        fi
        echo "- Root: \`${root:-none}\`"
        echo "- Units: \`$UNITS\`"
        echo "- State: \`$STATE\` (ledger + roots + evidence — independently re-verifiable)"
        echo "- Re-verify with: \`$BIN verify --state $STATE\`"
    } > "$rec_dir/test-record.md"
    echo "array-test-phase: wrote $rec_dir/test-record.md"
fi

if [ "$phase_ok" = "1" ]; then
    echo "array-test-phase: TEST PHASE GREEN"
    exit 0
else
    echo "array-test-phase: TEST PHASE RED" >&2
    exit 1
fi
