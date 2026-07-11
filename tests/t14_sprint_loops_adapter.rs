//! T14 — the sprint-loops Test-phase adapter (`adapters/sprint-loops/array-test-phase.sh`).
//! The adapter is a thin shell shim, so it is exercised as a black box: a green project
//! passes the phase and records it; a broken unit fails it.

#![cfg(unix)]

use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::tempdir;

fn adapter() -> PathBuf {
    Path::new(env!("CARGO_MANIFEST_DIR")).join("adapters/sprint-loops/array-test-phase.sh")
}

/// Copy the committed quickstart units into `<project>/units` — a realistic accreting
/// workspace for the phase to certify.
fn stage_project(project: &Path) {
    let src = Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/quickstart/units");
    for id in ["greeting", "announcer", "contract-audit"] {
        let from = src.join(id);
        let to = project.join("units").join(id);
        copy_tree(&from, &to);
    }
}

fn copy_tree(from: &Path, to: &Path) {
    fs::create_dir_all(to).unwrap();
    for entry in fs::read_dir(from).unwrap() {
        let entry = entry.unwrap();
        let dst = to.join(entry.file_name());
        if entry.file_type().unwrap().is_dir() {
            copy_tree(&entry.path(), &dst);
        } else {
            fs::copy(entry.path(), &dst).unwrap();
        }
    }
}

fn run_phase(project: &Path, extra: &[&str]) -> std::process::Output {
    let mut cmd = Command::new("/bin/sh");
    cmd.arg(adapter())
        .arg("--project")
        .arg(project)
        .args(extra)
        .env("ARRAY_TEST_BIN", env!("CARGO_BIN_EXE_array-test"));
    cmd.output().unwrap()
}

#[test]
fn given_a_green_project_the_test_phase_should_pass_and_write_a_record() {
    let project = tempdir().unwrap();
    stage_project(project.path());

    let out = run_phase(project.path(), &["--sprint", "s1"]);

    assert!(
        out.status.success(),
        "phase should be green: {}",
        String::from_utf8_lossy(&out.stderr)
    );
    let record = fs::read_to_string(project.path().join("sprints/s1/test-record.md")).unwrap();
    assert!(record.contains("GREEN"), "record: {record}");
    assert!(
        record.contains("blake3:"),
        "record must carry the root: {record}"
    );
    // The durable state is present and independently re-verifiable.
    assert!(project.path().join(".array-test/state").is_dir());
}

#[test]
fn given_a_broken_unit_the_test_phase_should_fail_red() {
    let project = tempdir().unwrap();
    stage_project(project.path());
    // Break the greeting unit's assertion so its cell goes red.
    let manifest = project.path().join("units/greeting/manifest.toml");
    let text = fs::read_to_string(&manifest).unwrap();
    fs::write(&manifest, text.replace("hello, world", "GOODBYE")).unwrap();

    let out = run_phase(project.path(), &["--sprint", "s2"]);

    assert_eq!(
        out.status.code(),
        Some(1),
        "a broken unit must fail the phase"
    );
    let record = fs::read_to_string(project.path().join("sprints/s2/test-record.md")).unwrap();
    assert!(record.contains("RED"), "record: {record}");
}

#[test]
fn given_no_binary_the_phase_should_report_a_usage_error() {
    let project = tempdir().unwrap();
    stage_project(project.path());
    let out = Command::new("/bin/sh")
        .arg(adapter())
        .arg("--project")
        .arg(project.path())
        .env("ARRAY_TEST_BIN", "/no/such/array-test-binary")
        .output()
        .unwrap();
    assert_eq!(out.status.code(), Some(2));
}
