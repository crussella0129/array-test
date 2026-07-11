//! Acceptance checks AC43–AC44 (sprints/s5/sprint-plans/test-plan.md): the
//! self-hosting milestone. array-test runs its own T2 acceptance suite as a cell —
//! through the tap adapter — and certifies a green root over itself.
//!
//! The cell runs the prebuilt libtest binary directly rather than `cargo test`: cargo
//! holds the build-dir lock for its whole session, so an inner cargo would deadlock
//! against the outer one. The direct binary needs no PATH, HOME, or cargo at all —
//! strictly more hermetic.

#![cfg(unix)]

use array_test::ledger::DetStatus;
use array_test::round::{run_round, CellOutcomeKind};
use std::fs;
use std::path::PathBuf;
use std::process::Command;
use tempfile::tempdir;

/// Locate a sibling test binary in the same deps dir this test binary runs from.
fn find_test_binary(prefix: &str) -> PathBuf {
    let deps_dir = std::env::current_exe()
        .unwrap()
        .parent()
        .unwrap()
        .to_path_buf();
    let mut candidates: Vec<PathBuf> = fs::read_dir(&deps_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| {
            p.is_file()
                && p.extension().is_none()
                && p.file_name()
                    .and_then(|n| n.to_str())
                    .is_some_and(|n| n.starts_with(&format!("{prefix}-")))
        })
        .collect();
    candidates.sort_by_key(|p| fs::metadata(p).and_then(|m| m.modified()).ok());
    candidates
        .pop()
        .unwrap_or_else(|| panic!("no {prefix}-* binary next to {}", deps_dir.display()))
}

#[test]
fn given_our_own_t2_suite_as_a_cell_the_array_should_certify_itself_green() {
    let tap_bin = env!("CARGO_BIN_EXE_array-test");
    let t2_bin = find_test_binary("t2_dag_resolver");

    let ws = tempdir().unwrap();
    let unit = ws.path().join("self-t2");
    fs::create_dir_all(unit.join("src")).unwrap();
    fs::write(
        unit.join("src/main.txt"),
        "self-host: t2 dag resolver suite",
    )
    .unwrap();
    fs::write(unit.join("contract.toml"), "[io]\n").unwrap();
    fs::write(
        unit.join("manifest.toml"),
        format!(
            "id = \"self.t2\"\nsprint = 5\nversion = \"0.1.0\"\n\n[test]\n\
             command = [\"{tap_bin}\", \"tap\", \"--\", \"{}\", \"--test-threads=1\"]\n\
             timeout_secs = 120\n",
            t2_bin.display()
        ),
    )
    .unwrap();

    let state = tempdir().unwrap();

    // AC43 — the round executes the cell (twice, via the meta-check), does NOT
    // quarantine it, and certifies green.
    let r1 = run_round(ws.path(), state.path(), None, 0, None).unwrap();
    let cell = &r1.cells[0];
    assert_eq!(cell.kind, CellOutcomeKind::Executed);
    assert_eq!(
        cell.det_status,
        DetStatus::Pass,
        "self-host cell was not a clean Pass — if Quarantined, the tap adapter leaked \
         nondeterminism into the evidence"
    );
    assert!(r1.record.all_pass);

    // AC44a — the certificate withstands independent verification.
    let verify = Command::new(tap_bin)
        .args(["verify", "--state"])
        .arg(state.path())
        .output()
        .unwrap();
    assert!(
        verify.status.success(),
        "verify failed: {}",
        String::from_utf8_lossy(&verify.stderr)
    );

    // AC44b — self-hosted frontier economics: nothing changed, so round 2 reuses the
    // confirmation without re-running the suite, and the root is identical.
    let r2 = run_round(ws.path(), state.path(), None, 0, None).unwrap();
    assert_eq!(r2.reused(), 1);
    assert_eq!(r2.executed(), 0);
    assert_eq!(r1.record.root, r2.record.root);
}
