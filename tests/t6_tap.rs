//! Acceptance checks AC39–AC42 (sprints/s5/sprint-plans/test-plan.md), driven through
//! the real `array-test tap` subcommand with sh-simulated libtest output.

#![cfg(unix)]

use std::process::Command;

fn tap_over(script: &str) -> std::process::Output {
    Command::new(env!("CARGO_BIN_EXE_array-test"))
        .args(["tap", "--", "/bin/sh", "-c", script])
        .output()
        .unwrap()
}

#[test]
fn given_noisy_libtest_output_should_emit_clean_sorted_tap_and_exit_zero() {
    let script = "printf 'running 2 tests\\ntest z_test ... ok\\ntest a_test ... ok\\n\\
                  test result: ok. 2 passed; finished in 0.37s\\n'";
    let out = tap_over(script);

    assert!(out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert_eq!(
        stdout,
        "TAP version 13\n1..2\nok 1 - a_test\nok 2 - z_test\n"
    );
    assert!(out.stderr.is_empty());
}

#[test]
fn given_a_failed_test_should_emit_not_ok_and_exit_nonzero() {
    let script = "printf 'test good ... ok\\ntest bad ... FAILED\\n'; exit 101";
    let out = tap_over(script);

    assert!(!out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("not ok 1 - bad"));
    assert!(stdout.contains("ok 2 - good"));
}

#[test]
fn given_ignored_tests_and_silent_failures_should_stay_honest() {
    // ignored -> SKIP
    let out = tap_over("printf 'test skipped_one ... ignored\\n'");
    assert!(String::from_utf8_lossy(&out.stdout).contains("ok 1 - skipped_one # SKIP"));

    // nonzero exit with no parsed failure -> synthetic not ok, nonzero exit
    let out = tap_over("printf 'test fine ... ok\\n'; exit 1");
    assert!(!out.status.success());
    let stdout = String::from_utf8_lossy(&out.stdout);
    assert!(stdout.contains("1..2"));
    assert!(stdout.contains("not ok 2 - inner process exited nonzero"));
}

#[test]
fn given_runs_differing_only_in_timing_noise_should_produce_identical_evidence() {
    // Same tests, different timing lines — the raw output differs, the TAP must not.
    let a = tap_over("printf 'test t ... ok\\ntest result: ok. finished in 0.11s\\n'");
    let b = tap_over("printf 'test t ... ok\\ntest result: ok. finished in 7.93s\\n'");

    assert_eq!(a.stdout, b.stdout);
    assert!(a.status.success() && b.status.success());
}
