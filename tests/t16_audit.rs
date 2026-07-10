//! Acceptance checks AC65–AC69 (sprints/s8/sprint-plans/test-plan.md): the full-audit
//! verifier and the committed quickstart example.

#![cfg(unix)]

use array_test::audit::full_audit;
use array_test::judge::{load_judge_config, run_with_judgment};
use array_test::round::{run_round, StatePaths};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use tempfile::{tempdir, TempDir};

/// A judged, repaired, multi-round state — every auditable surface populated.
fn rich_state() -> (TempDir, TempDir) {
    let ws = tempdir().unwrap();
    let unit = ws.path().join("u");
    fs::create_dir_all(unit.join("src")).unwrap();
    fs::write(unit.join("src/main.txt"), "starts bad").unwrap();
    fs::write(unit.join("contract.toml"), "[io]\n").unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "id = \"u\"\nsprint = 8\nversion = \"0.1.0\"\n\n\
         [test]\ncommand = [\"/bin/sh\", \"-c\", \"printf det-ok\"]\n",
    )
    .unwrap();
    fs::write(
        ws.path().join("judge.toml"),
        "command = [\"/bin/sh\", \"-c\", \"\
         if grep -q good \\\"$ARRAY_TEST_UNIT_DIR/src/main.txt\\\"; then \
           echo fine; echo 'rating: 95'; else echo needs-work; echo 'rating: 10'; fi\"]\n\
         runs = 2\nthreshold = 100\nmin_rating = 80\n\
         [repair]\ncommand = [\"/bin/sh\", \"-c\", \"printf good > \\\"$ARRAY_TEST_UNIT_DIR/src/main.txt\\\"\"]\nbudget = 1\n",
    )
    .unwrap();

    let state = tempdir().unwrap();
    let config = load_judge_config(ws.path()).unwrap().unwrap();
    let outcome = run_with_judgment(ws.path(), state.path(), 0, None, &config).unwrap();
    assert!(outcome.green, "fixture should converge green");
    (ws, state)
}

#[test]
fn given_a_healthy_judged_state_the_full_audit_should_pass_and_cli_verify_exit_zero() {
    let (_ws, state) = rich_state();

    let report = full_audit(state.path());

    assert!(report.clean(), "problems: {:?}", report.problems);
    assert!(report.confirmations >= 2, "two rounds expected");
    assert!(report.roots_checked >= 2);
    assert!(report.judgments >= 2, "rejection + acceptance");
    assert!(report.evidence_files >= 1);

    let verify = Command::new(env!("CARGO_BIN_EXE_array-test"))
        .args(["verify", "--state"])
        .arg(state.path())
        .output()
        .unwrap();
    assert!(verify.status.success());
    assert!(String::from_utf8_lossy(&verify.stdout).contains("VERIFIED"));
}

fn cli_verify_fails(state: &Path) -> bool {
    !Command::new(env!("CARGO_BIN_EXE_array-test"))
        .args(["verify", "--state"])
        .arg(state)
        .output()
        .unwrap()
        .status
        .success()
}

#[test]
fn given_a_tampered_root_certificate_the_audit_should_fail_despite_an_intact_ledger() {
    let (_ws, state) = rich_state();
    let paths = StatePaths::new(state.path());

    // Note: R1's certificate honestly reads all_pass=true — the DET phase was green in
    // round 1; it was the judge that rejected the cell, and root certificates are
    // Phase-D-only by design (D7). So the forgery to test is the root hash itself.
    let root_path = paths.roots_dir.join("R1.json");
    let text = fs::read_to_string(&root_path).unwrap();
    let tampered = if text.contains("blake3:9") {
        text.replace("blake3:9", "blake3:a")
    } else {
        text.replace("blake3:", "blake3:9")
    };
    assert_ne!(text, tampered);
    fs::write(&root_path, tampered).unwrap();

    let report = full_audit(state.path());
    assert!(!report.clean());
    assert!(report.problems.iter().any(|p| p.contains("R1")));
    assert!(cli_verify_fails(state.path()));
}

#[test]
fn given_a_tampered_judgment_line_the_audit_should_fail() {
    let (_ws, state) = rich_state();
    let paths = StatePaths::new(state.path());

    let text = fs::read_to_string(&paths.judgments_file).unwrap();
    let tampered = text.replace("\"verdict\":false", "\"verdict\":true");
    assert_ne!(text, tampered);
    fs::write(&paths.judgments_file, tampered).unwrap();

    let report = full_audit(state.path());
    assert!(!report.clean());
    assert!(report.problems.iter().any(|p| p.contains("judgments")));
    assert!(cli_verify_fails(state.path()));
}

#[test]
fn given_a_tampered_evidence_file_the_audit_should_fail() {
    let (_ws, state) = rich_state();
    let paths = StatePaths::new(state.path());

    let file = fs::read_dir(&paths.evidence_dir)
        .unwrap()
        .filter_map(|e| e.ok().map(|e| e.path()))
        .next()
        .expect("evidence stored");
    let mut bytes = fs::read(&file).unwrap();
    bytes[0] ^= 0xFF;
    fs::write(&file, bytes).unwrap();

    let report = full_audit(state.path());
    assert!(!report.clean());
    assert!(report
        .problems
        .iter()
        .any(|p| p.contains("content address")));
    assert!(cli_verify_fails(state.path()));
}

#[test]
fn given_the_committed_quickstart_example_a_round_should_run_green() {
    let units = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/quickstart/units");
    let state = tempdir().unwrap();

    let r1 = run_round(&units, state.path(), None, 0, None).unwrap();
    assert!(r1.record.all_pass, "quickstart must stay green: {r1:?}");
    assert_eq!(r1.record.cells, 2);
    assert_eq!(r1.executed(), 2);

    // And its README's central claim holds: an unchanged second round reuses all.
    let r2 = run_round(&units, state.path(), None, 0, None).unwrap();
    assert_eq!(r2.executed(), 0);
    assert_eq!(r2.record.root, r1.record.root);

    assert!(full_audit(state.path()).clean());
}
