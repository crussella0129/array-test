//! Acceptance checks AC85–AC89 (sprints/s13/sprint-plans/test-plan.md): per-unit
//! fixtures re-keying and the fuzz tier with a scripted fuzzer.

#![cfg(unix)]

use array_test::audit::full_audit;
use array_test::fuzz::{load_fuzz_config, read_fuzz_entries, run_fuzz};
use array_test::round::{run_round, CellOutcomeKind, StatePaths};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

/// The unit's test consumes its corpus: it fails if any corpus file contains "crash".
fn write_unit(units_dir: &Path, id: &str, src: &str) {
    let dir = units_dir.join(id);
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.txt"), src).unwrap();
    fs::write(dir.join("contract.toml"), "[io]\n").unwrap();
    fs::write(
        dir.join("manifest.toml"),
        format!(
            "id = \"{id}\"\nversion = \"1.0.0\"\n\n[test]\ncommand = [\"/bin/sh\", \"-c\", \
             \"! grep -rq crash fixtures/ 2>/dev/null\"]\n"
        ),
    )
    .unwrap();
}

/// Scripted fuzzer: drops a marker per invocation; "finds a crash" iff the unit's src
/// contains the token BUG (writing a crashing input into the corpus, exit 65).
fn write_fuzzer(units_dir: &Path, markers: &Path) {
    fs::write(
        units_dir.join("fuzz.toml"),
        format!(
            "command = [\"/bin/sh\", \"-c\", \"\
             date +%s%N > {}/$$-$(date +%s%N); \
             if grep -q BUG \\\"$ARRAY_TEST_UNIT_DIR/src/main.txt\\\"; then \
               printf 'crash input' > \\\"$ARRAY_TEST_CORPUS_DIR/finding-0\\\"; exit 65; \
             fi\"]\nbudget_secs = 5\n",
            markers.display()
        ),
    )
    .unwrap();
}

fn markers(dir: &Path) -> usize {
    fs::read_dir(dir).map(|d| d.count()).unwrap_or(0)
}

#[test]
fn given_fixture_changes_cells_should_rekey_and_absent_fixtures_keep_the_sentinel() {
    let ws = tempdir().unwrap();
    write_unit(ws.path(), "u", "no bugs here");
    let state = tempdir().unwrap();

    let r1 = run_round(ws.path(), state.path(), None, 0, None).unwrap();
    assert!(r1.record.all_pass);

    // AC85a: no fixtures dir -> sentinel -> unchanged round reuses.
    let r2 = run_round(ws.path(), state.path(), None, 0, None).unwrap();
    assert_eq!(r2.executed(), 0);
    assert_eq!(r1.record.root, r2.record.root);

    // AC85b: adding a fixture re-keys the cell.
    fs::create_dir_all(ws.path().join("u/fixtures")).unwrap();
    fs::write(ws.path().join("u/fixtures/seed.txt"), "benign input").unwrap();
    let r3 = run_round(ws.path(), state.path(), None, 0, None).unwrap();
    assert_eq!(r3.executed(), 1);
    assert_ne!(r3.record.root, r1.record.root);

    // ...and changing fixture content re-keys again.
    fs::write(ws.path().join("u/fixtures/seed.txt"), "different input").unwrap();
    let r4 = run_round(ws.path(), state.path(), None, 0, None).unwrap();
    assert_eq!(r4.executed(), 1);
}

#[test]
fn given_a_buggy_unit_findings_should_grow_the_corpus_and_rekey_the_cells() {
    let ws = tempdir().unwrap();
    write_unit(ws.path(), "buggy", "contains a BUG marker");
    let marker_dir = tempdir().unwrap();
    write_fuzzer(ws.path(), marker_dir.path());
    let state = tempdir().unwrap();

    // Baseline round: green (corpus empty).
    let r1 = run_round(ws.path(), state.path(), None, 0, None).unwrap();
    assert!(r1.record.all_pass);

    // AC86: fuzz finds a crash, writes it into the corpus, records the sidecar entry.
    let config = load_fuzz_config(ws.path()).unwrap().unwrap();
    let report = run_fuzz(ws.path(), state.path(), 0, &config).unwrap();
    assert!(!report.clean);
    assert!(report.units[0].record.findings);
    assert_ne!(
        report.units[0].record.fixtures_before,
        report.units[0].record.fixtures_after
    );
    assert!(ws.path().join("buggy/fixtures/fuzz/finding-0").exists());

    // The next round re-executes against the grown corpus — and now catches the crash
    // input (the test greps the corpus), going red until the "bug" is fixed.
    let r2 = run_round(ws.path(), state.path(), None, 0, None).unwrap();
    let cell = &r2.cells[0];
    assert_eq!(cell.kind, CellOutcomeKind::Executed);
    assert!(!r2.record.all_pass);
}

#[test]
fn given_a_clean_unit_results_should_cache_until_src_or_corpus_changes() {
    let ws = tempdir().unwrap();
    write_unit(ws.path(), "clean", "nothing wrong");
    let marker_dir = tempdir().unwrap();
    write_fuzzer(ws.path(), marker_dir.path());
    let state = tempdir().unwrap();
    let config = load_fuzz_config(ws.path()).unwrap().unwrap();

    let first = run_fuzz(ws.path(), state.path(), 0, &config).unwrap();
    assert!(first.clean);
    assert!(!first.units[0].cached);
    let baseline = markers(marker_dir.path());
    assert_eq!(baseline, 1);

    // AC87: unchanged -> cached, no invocation.
    let second = run_fuzz(ws.path(), state.path(), 0, &config).unwrap();
    assert!(second.units[0].cached);
    assert_eq!(markers(marker_dir.path()), baseline);

    // Changing src re-fuzzes.
    fs::write(ws.path().join("clean/src/main.txt"), "still nothing wrong").unwrap();
    let third = run_fuzz(ws.path(), state.path(), 0, &config).unwrap();
    assert!(!third.units[0].cached);
    assert!(markers(marker_dir.path()) > baseline);
}

#[test]
fn given_the_fuzz_sidecar_it_should_chain_verify_and_detect_tampering() {
    let ws = tempdir().unwrap();
    write_unit(ws.path(), "buggy", "contains a BUG marker");
    let marker_dir = tempdir().unwrap();
    write_fuzzer(ws.path(), marker_dir.path());
    let state = tempdir().unwrap();
    let config = load_fuzz_config(ws.path()).unwrap().unwrap();
    run_fuzz(ws.path(), state.path(), 0, &config).unwrap();

    let paths = StatePaths::new(state.path());
    let entries = read_fuzz_entries(&paths).unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].outcome.findings);

    let audit = full_audit(state.path());
    assert!(audit.clean(), "problems: {:?}", audit.problems);
    assert_eq!(audit.fuzz_entries, 1);

    // AC88: flip the recorded verdict.
    let text = fs::read_to_string(&paths.fuzz_file).unwrap();
    let tampered = text.replace("\"findings\":true", "\"findings\":false");
    assert_ne!(text, tampered);
    fs::write(&paths.fuzz_file, tampered).unwrap();
    let audit = full_audit(state.path());
    assert!(!audit.clean());
    assert!(audit.problems.iter().any(|p| p.contains("fuzz")));
}

#[test]
fn given_the_cli_fuzz_verb_exit_codes_should_track_findings() {
    let ws = tempdir().unwrap();
    write_unit(ws.path(), "clean", "nothing wrong");
    let marker_dir = tempdir().unwrap();
    write_fuzzer(ws.path(), marker_dir.path());
    let state = tempdir().unwrap();

    let run = |ws: &Path, state: &Path| {
        Command::new(env!("CARGO_BIN_EXE_array-test"))
            .args(["fuzz", "--units"])
            .arg(ws)
            .arg("--state")
            .arg(state)
            .output()
            .unwrap()
    };

    let out = run(ws.path(), state.path());
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("FUZZ CLEAN"));

    write_unit(ws.path(), "buggy", "has a BUG in it");
    let out = run(ws.path(), state.path());
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains("FUZZ FINDINGS"));
}
