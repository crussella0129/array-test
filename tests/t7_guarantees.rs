//! Acceptance checks AC56–AC58 (sprints/s7/sprint-plans/test-plan.md): guarantee
//! levels, the content-addressed evidence store, and a real Hypothesis property cell.

#![cfg(unix)]

use array_test::hash::{domain, Hash};
use array_test::ledger::{load_and_verify, DetStatus, Guarantee};
use array_test::round::{run_round, StatePaths};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn write_unit(units_dir: &Path, id: &str, manifest_tail: &str) {
    let dir = units_dir.join(id);
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.txt"), format!("content of {id}")).unwrap();
    fs::write(dir.join("contract.toml"), "[io]\n").unwrap();
    fs::write(
        dir.join("manifest.toml"),
        format!("id = \"{id}\"\nsprint = 7\nversion = \"0.1.0\"\n\n{manifest_tail}"),
    )
    .unwrap();
}

#[test]
fn given_a_guarantee_declaration_should_validate_record_and_rekey() {
    let ws = tempdir().unwrap();
    write_unit(
        ws.path(),
        "a",
        "[test]\ncommand = [\"/bin/sh\", \"-c\", \"printf ok\"]\nguarantee = \"property\"\n",
    );
    let state = tempdir().unwrap();

    let r1 = run_round(ws.path(), state.path(), None, 0, None).unwrap();
    assert!(r1.record.all_pass);

    let paths = StatePaths::new(state.path());
    let entries = load_and_verify(&paths.ledger_file).unwrap();
    assert_eq!(entries[0].guarantee, Guarantee::Property);

    // Changing only the declared guarantee re-keys the cell.
    write_unit(
        ws.path(),
        "a",
        "[test]\ncommand = [\"/bin/sh\", \"-c\", \"printf ok\"]\nguarantee = \"example\"\n",
    );
    let r2 = run_round(ws.path(), state.path(), None, 0, None).unwrap();
    assert_eq!(r2.executed(), 1);

    // Unknown guarantee is rejected at load.
    write_unit(
        ws.path(),
        "a",
        "[test]\ncommand = [\"/bin/sh\", \"-c\", \"printf ok\"]\nguarantee = \"vibes\"\n",
    );
    assert!(run_round(ws.path(), state.path(), None, 0, None).is_err());
}

#[test]
fn given_an_executed_cell_its_evidence_should_be_stored_and_rehash_to_the_ledger_hash() {
    let ws = tempdir().unwrap();
    write_unit(
        ws.path(),
        "a",
        "[test]\ncommand = [\"/bin/sh\", \"-c\", \"printf stored-evidence\"]\n",
    );
    let state = tempdir().unwrap();

    run_round(ws.path(), state.path(), None, 0, None).unwrap();

    let paths = StatePaths::new(state.path());
    let entries = load_and_verify(&paths.ledger_file).unwrap();
    let evidence_hash = entries[0].evidence_hash;

    let stored = fs::read(
        paths
            .evidence_dir
            .join(format!("{}.evidence", evidence_hash.hex())),
    )
    .unwrap();
    // The stored bytes are the exact framed encoding the hash covers.
    assert_eq!(Hash::leaf(domain::EVIDENCE, &stored), evidence_hash);
    // And the cell's stdout is inside them.
    assert!(stored
        .windows(b"stored-evidence".len())
        .any(|w| w == b"stored-evidence"));
}

#[test]
fn given_a_real_hypothesis_property_cell_should_pass_deterministically() {
    let have_hypothesis = Command::new("python3")
        .args(["-c", "import hypothesis"])
        .status()
        .map(|s| s.success())
        .unwrap_or(false);
    if !have_hypothesis {
        eprintln!("python3/hypothesis unavailable; AC58 skipped on this host");
        return;
    }

    let ws = tempdir().unwrap();
    let unit = ws.path().join("prop");
    fs::create_dir_all(unit.join("src")).unwrap();
    // The unit under test: a tiny pure function...
    fs::write(
        unit.join("src/impl.py"),
        "def add(a, b):\n    return a + b\n",
    )
    .unwrap();
    fs::write(
        unit.join("contract.toml"),
        "[properties]\ncommutative = \"add(a, b) == add(b, a)\"\n",
    )
    .unwrap();
    // ...and a derandomized Hypothesis property suite emitting TAP (D8/D14: the
    // property tier is just another deterministic TAP-emitting command).
    fs::write(
        unit.join("src/prop_test.py"),
        r#"import sys, os
sys.path.insert(0, os.path.join(os.environ["PWD_UNIT"], "src"))
from hypothesis import given, settings, strategies as st
from impl import add

@settings(derandomize=True, max_examples=50, database=None)
@given(st.integers(), st.integers())
def commutative(a, b):
    assert add(a, b) == add(b, a)

@settings(derandomize=True, max_examples=50, database=None)
@given(st.integers())
def identity(a):
    assert add(a, 0) == a

points = []
for name, prop in [("commutative", commutative), ("identity", identity)]:
    try:
        prop()
        points.append((name, True))
    except Exception:
        points.append((name, False))

print("TAP version 13")
print(f"1..{len(points)}")
ok_all = True
for i, (name, passed) in enumerate(sorted(points), 1):
    print(("ok" if passed else "not ok") + f" {i} - {name}")
    ok_all = ok_all and passed
sys.exit(0 if ok_all else 1)
"#,
    )
    .unwrap();
    let python = which_python();
    fs::write(
        unit.join("manifest.toml"),
        format!(
            "id = \"prop\"\nsprint = 7\nversion = \"0.1.0\"\n\n[test]\n\
             command = [\"{python}\", \"src/prop_test.py\"]\nguarantee = \"property\"\n\
             timeout_secs = 60\n\n[test.env]\nPWD_UNIT = \"{}\"\n",
            unit.display()
        ),
    )
    .unwrap();

    let state = tempdir().unwrap();
    let report = run_round(ws.path(), state.path(), None, 0, None).unwrap();

    // Passed BOTH runs of the determinism meta-check (derandomize did its job), with
    // the property guarantee recorded.
    assert!(
        report.record.all_pass,
        "property cell did not pass: {report:?}"
    );
    let paths = StatePaths::new(state.path());
    let entries = load_and_verify(&paths.ledger_file).unwrap();
    assert_eq!(entries[0].guarantee, Guarantee::Property);
    assert_eq!(entries[0].det_status, DetStatus::Pass);
}

fn which_python() -> String {
    // The hermetic cell has no PATH; resolve the interpreter to an absolute path here.
    let out = Command::new("/bin/sh")
        .args(["-c", "command -v python3"])
        .output()
        .unwrap();
    String::from_utf8_lossy(&out.stdout).trim().to_string()
}
