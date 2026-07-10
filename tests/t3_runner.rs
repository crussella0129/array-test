//! Acceptance checks AC19–AC23 (sprints/s3/sprint-plans/test-plan.md). Unix-targeted:
//! cells are real subprocesses driven through /bin/sh.

#![cfg(unix)]

use array_test::hash::Hash;
use array_test::runner::{run_cell, run_cell_checked, CellSpec, RunStatus, Verdict};
use std::collections::BTreeMap;
use std::time::Duration;

fn sh(script: &str) -> CellSpec {
    CellSpec {
        cell_key: Hash::of(script.as_bytes()),
        command: vec!["/bin/sh".into(), "-c".into(), script.into()],
        cwd: std::env::temp_dir(),
        env: BTreeMap::new(),
        seed: 42,
        timeout: Duration::from_secs(5),
    }
}

#[test]
fn given_a_zero_exit_should_report_pass() {
    let outcome = run_cell(&sh("exit 0")).unwrap();
    assert_eq!(outcome.status, RunStatus::Pass);
}

#[test]
fn given_a_nonzero_exit_should_report_fail_with_code() {
    let outcome = run_cell(&sh("exit 3")).unwrap();
    assert_eq!(outcome.status, RunStatus::Fail { exit_code: Some(3) });
}

#[test]
fn given_identical_cells_should_produce_identical_evidence_hashes() {
    let a = run_cell(&sh("echo ok; echo diag >&2")).unwrap();
    let b = run_cell(&sh("echo ok; echo diag >&2")).unwrap();

    assert_eq!(a.evidence.stdout, b"ok\n");
    assert_eq!(a.evidence.stderr, b"diag\n");
    assert_eq!(a.evidence_hash, b.evidence_hash);
}

#[test]
fn given_different_output_should_produce_different_evidence_hashes() {
    let a = run_cell(&sh("echo one")).unwrap();
    let b = run_cell(&sh("echo two")).unwrap();
    assert_ne!(a.evidence_hash, b.evidence_hash);
}

#[test]
fn given_stderr_only_difference_should_change_evidence_hash() {
    // stderr is inside the evidence commitment by design (s3 research §3).
    let a = run_cell(&sh("echo same; echo x >&2")).unwrap();
    let b = run_cell(&sh("echo same; echo y >&2")).unwrap();
    assert_ne!(a.evidence_hash, b.evidence_hash);
}

#[test]
fn given_a_parent_env_var_should_not_leak_into_the_cell() {
    std::env::set_var("ARRAY_TEST_LEAK_CANARY", "leaked");
    let outcome = run_cell(&sh("printf '%s' \"${ARRAY_TEST_LEAK_CANARY:-clean}\"")).unwrap();
    std::env::remove_var("ARRAY_TEST_LEAK_CANARY");

    assert_eq!(outcome.evidence.stdout, b"clean");
}

#[test]
fn given_a_cell_should_see_seed_and_hygiene_env() {
    let outcome = run_cell(&sh("printf '%s|%s|%s' \"$ARRAY_TEST_SEED\" \"$TZ\" \"$LC_ALL\""))
        .unwrap();
    assert_eq!(outcome.evidence.stdout, b"42|UTC|C");
}

#[test]
fn given_declared_env_should_reach_the_cell() {
    let mut spec = sh("printf '%s' \"$DECLARED\"");
    spec.env.insert("DECLARED".into(), "present".into());
    let outcome = run_cell(&spec).unwrap();
    assert_eq!(outcome.evidence.stdout, b"present");
}

#[test]
fn given_a_cell_exceeding_its_envelope_should_be_killed_and_reported_timed_out() {
    let mut spec = sh("sleep 30");
    spec.timeout = Duration::from_millis(100);

    let wall = std::time::Instant::now();
    let outcome = run_cell(&spec).unwrap();

    assert_eq!(outcome.status, RunStatus::TimedOut);
    // The whole call must return promptly — killing only the shell while a grandchild
    // (sleep) holds the output pipes would hang evidence collection for 30s.
    assert!(wall.elapsed() < Duration::from_secs(5));
}

#[test]
fn given_a_deterministic_cell_the_meta_check_should_confirm_it() {
    let verdict = run_cell_checked(&sh("echo stable")).unwrap();
    match verdict {
        Verdict::Confirmed(outcome) => assert_eq!(outcome.status, RunStatus::Pass),
        Verdict::Quarantined { .. } => panic!("deterministic cell was quarantined"),
    }
}

#[test]
fn given_a_nondeterministic_cell_the_meta_check_should_quarantine_it() {
    let verdict = run_cell_checked(&sh("head -c 16 /dev/urandom")).unwrap();
    match verdict {
        Verdict::Quarantined { first, second } => assert_ne!(first, second),
        Verdict::Confirmed(_) => panic!("nondeterministic cell escaped quarantine"),
    }
}
