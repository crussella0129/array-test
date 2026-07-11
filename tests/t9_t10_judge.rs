//! Acceptance checks AC59–AC64 (sprints/s7/sprint-plans/test-plan.md): the Phase-J
//! judge gate and the repair micro-loop, driven by scripted (deterministic) judges.

#![cfg(unix)]

use array_test::judge::{load_judge_config, read_judgments, run_with_judgment};
use array_test::round::StatePaths;
use std::fs;
use std::path::Path;
use tempfile::{tempdir, TempDir};

/// A workspace with one det-green unit whose src the judge inspects.
fn workspace(src_content: &str) -> TempDir {
    let ws = tempdir().unwrap();
    let unit = ws.path().join("u");
    fs::create_dir_all(unit.join("src")).unwrap();
    fs::write(unit.join("src/main.txt"), src_content).unwrap();
    fs::write(unit.join("contract.toml"), "[io]\n").unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "id = \"u\"\nsprint = 7\nversion = \"0.1.0\"\n\n\
         [test]\ncommand = [\"/bin/sh\", \"-c\", \"printf det-ok\"]\n",
    )
    .unwrap();
    ws
}

/// A scripted judge: rates 95 if the unit's src contains "good", else 15 — with a
/// critique explaining itself. Also drops an invocation marker for cache observation.
fn write_judge(ws: &Path, markers_dir: &Path, extra: &str) {
    fs::write(
        ws.join("judge.toml"),
        format!(
            "command = [\"/bin/sh\", \"-c\", \"\
             date +%s%N > {markers}/$$-$(date +%s%N); \
             if grep -q good \\\"$ARRAY_TEST_UNIT_DIR/src/main.txt\\\"; then \
               echo 'The implementation matches the spirit of the contract.'; echo 'rating: 95'; \
             else \
               echo 'CRITIQUE: src/main.txt must contain the token good.'; echo 'rating: 15'; \
             fi\"]\nruns = 3\nthreshold = 100\nmin_rating = 80\n{extra}",
            markers = markers_dir.display()
        ),
    )
    .unwrap();
}

fn marker_count(markers_dir: &Path) -> usize {
    fs::read_dir(markers_dir)
        .map(std::iter::Iterator::count)
        .unwrap_or(0)
}

#[test]
fn given_a_low_rating_judge_a_det_green_round_should_be_judged_red_with_audit_trail() {
    let ws = workspace("this content is bad");
    let markers = tempdir().unwrap();
    write_judge(ws.path(), markers.path(), "");
    let state = tempdir().unwrap();
    let config = load_judge_config(ws.path()).unwrap().unwrap();

    let outcome = run_with_judgment(ws.path(), state.path(), 0, None, &config).unwrap();

    // Det green, judge red — the two-phase AND gate (D7).
    assert!(outcome.det.record.all_pass);
    assert!(!outcome.green);
    assert_eq!(outcome.judged.len(), 1);
    assert!(!outcome.judged[0].judgment.verdict);
    assert_eq!(outcome.judged[0].judgment.pass_runs, 0);

    // Critique transcript exists and says why.
    let paths = StatePaths::new(state.path());
    let critique = state.path().join(&outcome.judged[0].judgment.critique_ref);
    assert!(fs::read_to_string(critique).unwrap().contains("CRITIQUE"));

    // The judgments ledger chain-verifies and holds the verdict.
    let judgments = read_judgments(&paths).unwrap();
    assert_eq!(judgments.len(), 1);
    assert!(!judgments[0].judgment.verdict);

    // No repair configured -> failure record written (AC64, escalation half).
    let record = outcome.failure_record.expect("failure record expected");
    assert!(fs::read_to_string(record).unwrap().contains("rejected"));
}

#[test]
fn given_an_unchanged_cell_and_judge_the_verdict_should_be_cached_not_reinvoked() {
    let ws = workspace("all good here");
    let markers = tempdir().unwrap();
    write_judge(ws.path(), markers.path(), "");
    let state = tempdir().unwrap();
    let config = load_judge_config(ws.path()).unwrap().unwrap();

    let first = run_with_judgment(ws.path(), state.path(), 0, None, &config).unwrap();
    assert!(first.green);
    assert!(!first.judged[0].cached);
    let invocations_after_first = marker_count(markers.path());
    assert_eq!(invocations_after_first, 3, "runs = 3 judge passes");

    // Nothing changed: det reuses AND the judge verdict is cached — zero invocations.
    let second = run_with_judgment(ws.path(), state.path(), 0, None, &config).unwrap();
    assert!(second.green);
    assert!(second.judged[0].cached);
    assert_eq!(marker_count(markers.path()), invocations_after_first);
}

#[test]
fn given_a_changed_judge_config_the_same_cell_should_be_rejudged() {
    let ws = workspace("all good here");
    let markers = tempdir().unwrap();
    write_judge(ws.path(), markers.path(), "");
    let state = tempdir().unwrap();
    let config = load_judge_config(ws.path()).unwrap().unwrap();
    run_with_judgment(ws.path(), state.path(), 0, None, &config).unwrap();
    let baseline = marker_count(markers.path());

    // Same command, different min_rating -> new judge_hash (R-f) -> cache miss.
    write_judge(ws.path(), markers.path(), "");
    let mut config2 = load_judge_config(ws.path()).unwrap().unwrap();
    config2.min_rating = 90;
    let outcome = run_with_judgment(ws.path(), state.path(), 0, None, &config2).unwrap();

    assert!(!outcome.judged[0].cached);
    assert!(marker_count(markers.path()) > baseline);
}

#[test]
fn given_a_det_red_round_the_judge_should_never_be_invoked() {
    let ws = workspace("irrelevant");
    let unit = ws.path().join("u");
    fs::write(
        unit.join("manifest.toml"),
        "id = \"u\"\nsprint = 7\nversion = \"0.1.0\"\n\n\
         [test]\ncommand = [\"/bin/sh\", \"-c\", \"exit 1\"]\n",
    )
    .unwrap();
    let markers = tempdir().unwrap();
    write_judge(ws.path(), markers.path(), "");
    let state = tempdir().unwrap();
    let config = load_judge_config(ws.path()).unwrap().unwrap();

    let outcome = run_with_judgment(ws.path(), state.path(), 0, None, &config).unwrap();

    assert!(!outcome.green);
    assert!(!outcome.det.record.all_pass);
    assert!(outcome.judged.is_empty());
    assert_eq!(
        marker_count(markers.path()),
        0,
        "Phase J entered on a det-red round"
    );
}

#[test]
fn given_a_repair_command_a_rejected_unit_should_converge_across_rounds() {
    let ws = workspace("this content is bad");
    let markers = tempdir().unwrap();
    // Repair: replace the offending content, guided by (the existence of) the critique.
    write_judge(
        ws.path(),
        markers.path(),
        "[repair]\ncommand = [\"/bin/sh\", \"-c\", \"\
         test -f \\\"$ARRAY_TEST_CRITIQUE\\\" && \
         printf 'now good, per critique' > \\\"$ARRAY_TEST_UNIT_DIR/src/main.txt\\\"\"]\n\
         budget = 2\n",
    );
    let state = tempdir().unwrap();
    let config = load_judge_config(ws.path()).unwrap().unwrap();

    let outcome = run_with_judgment(ws.path(), state.path(), 0, None, &config).unwrap();

    // Converged: one repair attempt, then det re-ran the re-keyed cell and the judge
    // passed. Attempts are visible as ordinary rounds (R1 = rejected, R2 = green).
    assert!(outcome.green, "repair loop did not converge: {outcome:?}");
    assert_eq!(outcome.repair_attempts, 1);
    assert_eq!(outcome.det.record.round, 2);
    assert!(outcome.failure_record.is_none());

    // History: the confirmations ledger holds both rounds; the judgments ledger holds
    // the rejection and the acceptance.
    let paths = StatePaths::new(state.path());
    let judgments = read_judgments(&paths).unwrap();
    assert_eq!(judgments.len(), 2);
    assert!(!judgments[0].judgment.verdict);
    assert!(judgments[1].judgment.verdict);
}

#[test]
fn given_an_exhausted_budget_should_escalate_with_a_failure_record() {
    let ws = workspace("this content is bad");
    let markers = tempdir().unwrap();
    // A useless repairer: touches the file but never introduces "good" (still re-keys
    // each attempt, so the loop genuinely retries rather than reusing).
    write_judge(
        ws.path(),
        markers.path(),
        "[repair]\ncommand = [\"/bin/sh\", \"-c\", \"\
         date +%s%N >> \\\"$ARRAY_TEST_UNIT_DIR/src/main.txt\\\"\"]\nbudget = 2\n",
    );
    let state = tempdir().unwrap();
    let config = load_judge_config(ws.path()).unwrap().unwrap();

    let outcome = run_with_judgment(ws.path(), state.path(), 0, None, &config).unwrap();

    assert!(!outcome.green);
    assert_eq!(outcome.repair_attempts, 2);
    let record = outcome.failure_record.expect("escalation record expected");
    let body = fs::read_to_string(record).unwrap();
    assert!(body.contains("Phase J rejected"));
    assert!(body.contains("critique"));
}
