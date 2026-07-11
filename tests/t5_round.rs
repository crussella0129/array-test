//! Acceptance checks AC29–AC36 (sprints/s4/sprint-plans/test-plan.md): the first real
//! regression rounds, driven through the library API on deterministic sh-based cells.

#![cfg(unix)]

use array_test::ledger::{load_and_verify, DetStatus};
use array_test::round::{run_round, CellOutcomeKind, RoundReport, StatePaths};
use std::fs;
use std::path::Path;
use tempfile::{tempdir, TempDir};

/// Workspace fixture: chain a <- b <- c (b deps a, c deps b), all green by default.
fn write_unit(units_dir: &Path, id: &str, deps: &[&str], src: &str, script: &str) {
    let dir = units_dir.join(id);
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.txt"), src).unwrap();
    fs::write(dir.join("contract.toml"), "[io]\n").unwrap();
    let deps_toml = deps
        .iter()
        .map(|d| format!("\"{d}\""))
        .collect::<Vec<_>>()
        .join(", ");
    fs::write(
        dir.join("manifest.toml"),
        format!(
            "id = \"{id}\"\nsprint = 1\nversion = \"0.1.0\"\ndeps = [{deps_toml}]\n\n\
             [test]\ncommand = [\"/bin/sh\", \"-c\", \"{script}\"]\n"
        ),
    )
    .unwrap();
}

fn chain_workspace() -> TempDir {
    let ws = tempdir().unwrap();
    write_unit(ws.path(), "a", &[], "alpha v1", "printf a-ok");
    write_unit(ws.path(), "b", &["a"], "beta v1", "printf b-ok");
    write_unit(ws.path(), "c", &["b"], "gamma v1", "printf c-ok");
    ws
}

fn round(units: &Path, state: &Path) -> RoundReport {
    run_round(units, state, None, 0, None).unwrap()
}

fn executed_units(report: &RoundReport) -> Vec<&str> {
    report
        .cells
        .iter()
        .filter(|c| c.kind == CellOutcomeKind::Executed)
        .map(|c| c.unit_id.as_str())
        .collect()
}

#[test]
fn given_a_fresh_workspace_round_one_should_execute_everything_and_go_green() {
    let ws = chain_workspace();
    let state = tempdir().unwrap();

    let report = round(ws.path(), state.path());

    assert_eq!(report.record.round, 1);
    assert_eq!(report.record.cells, 3);
    assert_eq!(report.executed(), 3);
    assert_eq!(report.reused(), 0);
    assert!(report.record.all_pass);
    assert!(state.path().join("ledger/roots/R1.json").is_file());
}

#[test]
fn given_no_changes_round_two_should_reuse_everything_with_an_identical_root() {
    let ws = chain_workspace();
    let state = tempdir().unwrap();

    let r1 = round(ws.path(), state.path());
    let r2 = round(ws.path(), state.path());

    assert_eq!(r2.record.round, 2);
    assert_eq!(r2.executed(), 0);
    assert_eq!(r2.reused(), 3);
    assert!(r2.record.all_pass);
    // Same planned cells, same statuses -> byte-identical certificate root.
    assert_eq!(r1.record.root, r2.record.root);
}

#[test]
fn given_a_changed_leaf_unit_only_that_cell_should_execute() {
    let ws = chain_workspace();
    let state = tempdir().unwrap();
    round(ws.path(), state.path());

    // c is the leaf-most dependent: nothing depends on it.
    fs::write(ws.path().join("c/src/main.txt"), "gamma v2").unwrap();
    let r2 = round(ws.path(), state.path());

    assert_eq!(executed_units(&r2), vec!["c"]);
    assert_eq!(r2.reused(), 2);
}

#[test]
fn given_a_changed_root_dependency_the_full_dependent_closure_should_execute() {
    let ws = chain_workspace();
    let state = tempdir().unwrap();
    round(ws.path(), state.path());

    // a underpins b and c: the backwards arrow must re-run all three.
    fs::write(ws.path().join("a/src/main.txt"), "alpha v2").unwrap();
    let r2 = round(ws.path(), state.path());

    assert_eq!(r2.executed(), 3);
    assert_eq!(r2.reused(), 0);
}

#[test]
fn given_a_deterministic_failure_it_should_gate_the_round_and_then_be_reused() {
    let ws = chain_workspace();
    write_unit(ws.path(), "d", &[], "delta v1", "exit 1");
    let state = tempdir().unwrap();

    let r1 = round(ws.path(), state.path());
    assert!(!r1.record.all_pass);

    let r2 = round(ws.path(), state.path());
    let d = r2.cells.iter().find(|c| c.unit_id == "d").unwrap();
    // Nothing changed: the failure is a fact about this key, not something to re-check.
    assert_eq!(d.kind, CellOutcomeKind::Reused);
    assert_eq!(d.det_status, DetStatus::Fail);
    assert!(!r2.record.all_pass);
}

#[test]
fn given_a_nondeterministic_cell_it_should_quarantine_not_cache_and_not_go_green() {
    let ws = chain_workspace();
    write_unit(ws.path(), "e", &[], "epsilon v1", "head -c 8 /dev/urandom");
    let state = tempdir().unwrap();

    let r1 = round(ws.path(), state.path());
    let e1 = r1.cells.iter().find(|c| c.unit_id == "e").unwrap();
    assert_eq!(e1.det_status, DetStatus::Quarantined);
    assert!(!r1.record.all_pass);

    let r2 = round(ws.path(), state.path());
    let e2 = r2.cells.iter().find(|c| c.unit_id == "e").unwrap();
    // Quarantine never enters the cache: the cell runs (and fails to reproduce) again.
    assert_eq!(e2.kind, CellOutcomeKind::Executed);
    assert_eq!(e2.det_status, DetStatus::Quarantined);
}

#[test]
fn given_a_change_between_rounds_stale_keys_should_not_leak_into_the_new_certificate() {
    let ws = chain_workspace();
    let state = tempdir().unwrap();
    let r1 = round(ws.path(), state.path());

    fs::write(ws.path().join("a/src/main.txt"), "alpha v2").unwrap();
    let r2 = round(ws.path(), state.path());

    // Still exactly 3 planned cells — the old keys live in history, not the certificate.
    assert_eq!(r2.record.cells, 3);
    assert_ne!(r1.record.root, r2.record.root);

    // And the full ledger (both rounds, 6 entries) still chain-verifies.
    let paths = StatePaths::new(state.path());
    let entries = load_and_verify(&paths.ledger_file).unwrap();
    assert_eq!(entries.len(), 6);
}

#[test]
fn given_two_rounds_reused_entries_should_be_marked_in_the_ledger() {
    let ws = chain_workspace();
    let state = tempdir().unwrap();
    round(ws.path(), state.path());
    round(ws.path(), state.path());

    let paths = StatePaths::new(state.path());
    let entries = load_and_verify(&paths.ledger_file).unwrap();
    let (r1, r2): (Vec<_>, Vec<_>) = entries.iter().partition(|e| e.round == 1);

    assert!(r1.iter().all(|e| !e.reused));
    assert!(r2.iter().all(|e| e.reused));
}
