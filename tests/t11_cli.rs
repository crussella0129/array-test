//! Acceptance checks AC37–AC38 (sprints/s4/sprint-plans/test-plan.md): the standalone
//! binary an embedder drives (D11).

#![cfg(unix)]

use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn bin() -> Command {
    Command::new(env!("CARGO_BIN_EXE_array-test"))
}

fn write_unit(units_dir: &Path, id: &str, script: &str) {
    let dir = units_dir.join(id);
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.txt"), "content").unwrap();
    fs::write(dir.join("contract.toml"), "[io]\n").unwrap();
    fs::write(
        dir.join("manifest.toml"),
        format!(
            "id = \"{id}\"\nsprint = 1\nversion = \"0.1.0\"\n\n\
             [test]\ncommand = [\"/bin/sh\", \"-c\", \"{script}\"]\n"
        ),
    )
    .unwrap();
}

#[test]
fn given_a_green_workspace_run_should_exit_zero_and_write_the_certificate() {
    let ws = tempdir().unwrap();
    let state = tempdir().unwrap();
    write_unit(ws.path(), "a", "printf ok");

    let output = bin()
        .args(["run", "--units"])
        .arg(ws.path())
        .arg("--state")
        .arg(state.path())
        .output()
        .unwrap();

    assert!(
        output.status.success(),
        "stdout: {stdout}",
        stdout = String::from_utf8_lossy(&output.stdout)
    );
    assert!(String::from_utf8_lossy(&output.stdout).contains("ALL PASS"));
    assert!(state.path().join("ledger/roots/R1.json").is_file());
}

#[test]
fn given_a_red_workspace_run_should_exit_one() {
    let ws = tempdir().unwrap();
    let state = tempdir().unwrap();
    write_unit(ws.path(), "a", "exit 1");

    let output = bin()
        .args(["run", "--units"])
        .arg(ws.path())
        .arg("--state")
        .arg(state.path())
        .output()
        .unwrap();

    assert_eq!(output.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&output.stdout).contains("NOT GREEN"));
}

#[test]
fn given_an_intact_state_verify_should_exit_zero_and_after_tampering_nonzero() {
    let ws = tempdir().unwrap();
    let state = tempdir().unwrap();
    write_unit(ws.path(), "a", "printf ok");

    let run = bin()
        .args(["run", "--units"])
        .arg(ws.path())
        .arg("--state")
        .arg(state.path())
        .output()
        .unwrap();
    assert!(run.status.success());

    let verify = bin()
        .args(["verify", "--state"])
        .arg(state.path())
        .output()
        .unwrap();
    assert!(verify.status.success());

    // Flip the recorded status without recomputing hashes.
    let ledger = state.path().join("ledger/confirmations.ndjson");
    let text = fs::read_to_string(&ledger).unwrap();
    let tampered = text.replace("\"pass\"", "\"fail\"");
    assert_ne!(text, tampered);
    fs::write(&ledger, tampered).unwrap();

    let verify_tampered = bin()
        .args(["verify", "--state"])
        .arg(state.path())
        .output()
        .unwrap();
    assert!(!verify_tampered.status.success());
}
