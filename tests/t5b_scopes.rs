//! Acceptance checks AC45–AC51 (sprints/s6/sprint-plans/test-plan.md): the scope
//! ladder — per-scope key semantics, tier gating with visible Skipped, and back-compat.

#![cfg(unix)]

use array_test::hash::CellScope;
use array_test::ledger::{load_and_verify, DetStatus};
use array_test::round::{run_round, CellOutcomeKind, RoundReport, StatePaths};
use std::fs;
use std::path::Path;
use tempfile::{tempdir, TempDir};

fn write_unit_with(units_dir: &Path, id: &str, deps: &[&str], src: &str, tests_toml: &str) {
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
            "id = \"{id}\"\nsprint = 6\nversion = \"0.1.0\"\ndeps = [{deps_toml}]\n\n{tests_toml}"
        ),
    )
    .unwrap();
}

fn ok_test(scope: &str) -> String {
    format!("[tests.{scope}]\ncommand = [\"/bin/sh\", \"-c\", \"printf {scope}-ok\"]\n")
}

fn round(units: &Path, state: &Path) -> RoundReport {
    run_round(units, state, None, 0, None).unwrap()
}

fn cell<'r>(
    report: &'r RoundReport,
    unit: &str,
    scope: CellScope,
) -> &'r array_test::round::CellReport {
    report
        .cells
        .iter()
        .find(|c| c.unit_id == unit && c.scope == scope)
        .unwrap_or_else(|| panic!("no cell for {unit}@{}", scope.as_str()))
}

/// Chain a <- b <- c. b carries unit+closure tests; c carries a direct test.
fn ladder_workspace() -> TempDir {
    let ws = tempdir().unwrap();
    write_unit_with(ws.path(), "a", &[], "alpha v1", &ok_test("closure"));
    write_unit_with(
        ws.path(),
        "b",
        &["a"],
        "beta v1",
        &format!("{}{}", ok_test("unit"), ok_test("closure")),
    );
    write_unit_with(ws.path(), "c", &["b"], "gamma v1", &ok_test("direct"));
    ws
}

#[test]
fn given_a_dep_change_a_unit_scope_cell_should_stay_reused_while_closure_reruns() {
    let ws = ladder_workspace();
    let state = tempdir().unwrap();
    round(ws.path(), state.path());

    fs::write(ws.path().join("a/src/main.txt"), "alpha v2").unwrap();
    let r2 = round(ws.path(), state.path());

    // b's unit-scope key excludes deps: unaffected by a. Its closure-scope key isn't.
    assert_eq!(cell(&r2, "b", CellScope::Unit).kind, CellOutcomeKind::Reused);
    assert_eq!(
        cell(&r2, "b", CellScope::Closure).kind,
        CellOutcomeKind::Executed
    );
}

#[test]
fn given_a_transitive_change_a_direct_scope_cell_should_stay_reused() {
    let ws = ladder_workspace();
    let state = tempdir().unwrap();
    round(ws.path(), state.path());

    // a is transitive (not direct) for c: c's direct-scope cell must not re-key.
    fs::write(ws.path().join("a/src/main.txt"), "alpha v2").unwrap();
    let r2 = round(ws.path(), state.path());
    assert_eq!(
        cell(&r2, "c", CellScope::Direct).kind,
        CellOutcomeKind::Reused
    );

    // b IS direct for c: now it must re-key.
    fs::write(ws.path().join("b/src/main.txt"), "beta v2").unwrap();
    let r3 = round(ws.path(), state.path());
    assert_eq!(
        cell(&r3, "c", CellScope::Direct).kind,
        CellOutcomeKind::Executed
    );
}

#[test]
fn given_any_change_anywhere_an_e2e_cell_should_rerun() {
    let ws = ladder_workspace();
    // e is an entrypoint with no deps at all — e2e still keys over the whole workspace.
    write_unit_with(ws.path(), "e", &[], "entry v1", &ok_test("e2e"));
    let state = tempdir().unwrap();
    round(ws.path(), state.path());

    fs::write(ws.path().join("a/src/main.txt"), "alpha v2").unwrap();
    let r2 = round(ws.path(), state.path());

    assert_eq!(
        cell(&r2, "e", CellScope::E2e).kind,
        CellOutcomeKind::Executed
    );
}

#[test]
fn given_a_failing_unit_tier_cell_higher_tiers_should_be_skipped_but_siblings_run() {
    let ws = ladder_workspace();
    write_unit_with(
        ws.path(),
        "d",
        &[],
        "delta v1",
        "[tests.unit]\ncommand = [\"/bin/sh\", \"-c\", \"exit 1\"]\n",
    );
    let state = tempdir().unwrap();

    let r1 = round(ws.path(), state.path());

    // d's unit cell failed; b's unit cell is a same-tier sibling and still ran.
    assert_eq!(cell(&r1, "d", CellScope::Unit).det_status, DetStatus::Fail);
    assert_eq!(
        cell(&r1, "b", CellScope::Unit).kind,
        CellOutcomeKind::Executed
    );
    // Every higher-tier cell is Skipped — visible in the report and the ledger.
    for c in r1.cells.iter().filter(|c| c.scope != CellScope::Unit) {
        assert_eq!(c.det_status, DetStatus::Skipped, "{}@{:?}", c.unit_id, c.scope);
        assert_eq!(c.kind, CellOutcomeKind::Skipped);
    }
    assert!(!r1.record.all_pass);

    let paths = StatePaths::new(state.path());
    let entries = load_and_verify(&paths.ledger_file).unwrap();
    assert!(entries.iter().any(|e| e.det_status == DetStatus::Skipped));
}

#[test]
fn given_the_gate_lifts_previously_skipped_cells_should_execute_not_reuse() {
    let ws = ladder_workspace();
    write_unit_with(
        ws.path(),
        "d",
        &[],
        "delta v1",
        "[tests.unit]\ncommand = [\"/bin/sh\", \"-c\", \"exit 1\"]\n",
    );
    let state = tempdir().unwrap();
    round(ws.path(), state.path());

    // Fix d: gate lifts; the closure cells were Skipped (never cached) so they execute.
    write_unit_with(
        ws.path(),
        "d",
        &[],
        "delta v2",
        "[tests.unit]\ncommand = [\"/bin/sh\", \"-c\", \"printf fixed\"]\n",
    );
    let r2 = round(ws.path(), state.path());

    assert!(r2.record.all_pass);
    assert_eq!(
        cell(&r2, "a", CellScope::Closure).kind,
        CellOutcomeKind::Executed
    );
    assert_eq!(r2.skipped(), 0);
}

#[test]
fn given_legacy_test_and_tests_closure_together_should_be_rejected() {
    let ws = tempdir().unwrap();
    write_unit_with(
        ws.path(),
        "x",
        &[],
        "src",
        "[test]\ncommand = [\"/bin/true\"]\n\n[tests.closure]\ncommand = [\"/bin/true\"]\n",
    );
    let state = tempdir().unwrap();

    assert!(run_round(ws.path(), state.path(), None, 0, None).is_err());
}

#[test]
fn given_an_unknown_scope_should_be_rejected_at_load() {
    let ws = tempdir().unwrap();
    write_unit_with(
        ws.path(),
        "x",
        &[],
        "src",
        "[tests.galactic]\ncommand = [\"/bin/true\"]\n",
    );
    let state = tempdir().unwrap();

    assert!(run_round(ws.path(), state.path(), None, 0, None).is_err());
}
