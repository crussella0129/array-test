//! Acceptance checks AC79–AC84 (sprints/s12/sprint-plans/test-plan.md): the mutation
//! tier, driven by a scripted deterministic mutator.

#![cfg(unix)]

use array_test::audit::full_audit;
use array_test::mutation::{load_mutation_config, read_mutations, run_mutation};
use array_test::round::StatePaths;
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::{tempdir, TempDir};

/// `checked` really tests its content; `weak` asserts nothing — the exact pathology
/// mutation testing exists to expose.
fn write_unit(units_dir: &Path, id: &str, test_cmd: &str) {
    let dir = units_dir.join(id);
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/data.txt"), format!("magic token of {id}\n")).unwrap();
    fs::write(dir.join("contract.toml"), "[io]\n").unwrap();
    fs::write(
        dir.join("manifest.toml"),
        format!(
            "id = \"{id}\"\nversion = \"1.0.0\"\n\n\
             [test]\ncommand = [\"/bin/sh\", \"-c\", \"{test_cmd}\"]\n"
        ),
    )
    .unwrap();
}

/// Mutator: corrupts src/data.txt deterministically per index, drops an invocation
/// marker so caching is observable.
fn write_mutator(units_dir: &Path, markers: &Path) {
    fs::write(
        units_dir.join("mutation.toml"),
        format!(
            "command = [\"/bin/sh\", \"-c\", \"\
             date +%s%N > {}/$$-$(date +%s%N); \
             printf 'corrupted %s' \\\"$ARRAY_TEST_MUTANT_INDEX\\\" > \\\"$ARRAY_TEST_UNIT_DIR/src/data.txt\\\"\"]\n\
             mutants = 2\nmin_score = 100\n",
            markers.display()
        ),
    )
    .unwrap();
}

fn markers(dir: &Path) -> usize {
    fs::read_dir(dir).map(|d| d.count()).unwrap_or(0)
}

#[test]
fn given_a_real_test_all_mutants_die_and_a_vacuous_test_lets_them_all_live() {
    let ws = tempdir().unwrap();
    write_unit(ws.path(), "checked", "grep -q magic src/data.txt");
    write_unit(ws.path(), "weak", "true");
    let marker_dir = tempdir().unwrap();
    write_mutator(ws.path(), marker_dir.path());
    let state = tempdir().unwrap();
    let config = load_mutation_config(ws.path()).unwrap().unwrap();

    let report = run_mutation(ws.path(), state.path(), 0, None, &config).unwrap();

    let checked = report
        .units
        .iter()
        .find(|u| u.score.unit_id == "checked")
        .unwrap();
    assert_eq!(checked.score.killed, 2);
    assert_eq!(checked.score.score_pct, 100);
    assert!(checked.score.strong);

    let weak = report
        .units
        .iter()
        .find(|u| u.score.unit_id == "weak")
        .unwrap();
    assert_eq!(weak.score.killed, 0);
    assert_eq!(weak.score.score_pct, 0);
    assert!(!weak.score.strong);

    assert!(!report.all_strong);
}

#[test]
fn given_an_unchanged_workspace_scores_should_be_served_from_cache() {
    let ws = tempdir().unwrap();
    write_unit(ws.path(), "checked", "grep -q magic src/data.txt");
    let marker_dir = tempdir().unwrap();
    write_mutator(ws.path(), marker_dir.path());
    let state = tempdir().unwrap();
    let config = load_mutation_config(ws.path()).unwrap().unwrap();

    let first = run_mutation(ws.path(), state.path(), 0, None, &config).unwrap();
    assert!(first.all_strong);
    assert!(!first.units[0].cached);
    let after_first = markers(marker_dir.path());
    assert_eq!(after_first, 2, "two mutants requested");

    // AC81: nothing changed — zero mutator invocations, score cached.
    let second = run_mutation(ws.path(), state.path(), 0, None, &config).unwrap();
    assert!(second.units[0].cached);
    assert_eq!(markers(marker_dir.path()), after_first);

    // AC82: changing the unit (still green: magic kept) re-keys the detection surface
    // and re-mutates.
    fs::write(
        ws.path().join("checked/src/data.txt"),
        "magic token, second edition\n",
    )
    .unwrap();
    let third = run_mutation(ws.path(), state.path(), 0, None, &config).unwrap();
    assert!(!third.units[0].cached);
    assert!(markers(marker_dir.path()) > after_first);
}

#[test]
fn given_the_mutations_sidecar_it_should_chain_verify_and_detect_tampering() {
    let ws = tempdir().unwrap();
    write_unit(ws.path(), "checked", "grep -q magic src/data.txt");
    let marker_dir = tempdir().unwrap();
    write_mutator(ws.path(), marker_dir.path());
    let state = tempdir().unwrap();
    let config = load_mutation_config(ws.path()).unwrap().unwrap();
    run_mutation(ws.path(), state.path(), 0, None, &config).unwrap();

    let paths = StatePaths::new(state.path());
    let entries = read_mutations(&paths).unwrap();
    assert_eq!(entries.len(), 1);
    assert!(entries[0].score.strong);

    let audit = full_audit(state.path());
    assert!(audit.clean(), "problems: {:?}", audit.problems);
    assert_eq!(audit.mutations, 1);

    // AC83: flip the recorded verdict without recomputing hashes.
    let text = fs::read_to_string(&paths.mutations_file).unwrap();
    let tampered = text.replace("\"strong\":true", "\"strong\":false");
    assert_ne!(text, tampered);
    fs::write(&paths.mutations_file, tampered).unwrap();

    let audit = full_audit(state.path());
    assert!(!audit.clean());
    assert!(audit.problems.iter().any(|p| p.contains("mutation")));
}

fn cli(args: &[&str], extra: &[&std::ffi::OsStr]) -> std::process::Output {
    let mut cmd = Command::new(env!("CARGO_BIN_EXE_array-test"));
    cmd.args(args);
    for a in extra {
        cmd.arg(a);
    }
    cmd.output().unwrap()
}

#[test]
fn given_the_cli_mutate_verb_exit_codes_should_track_all_strong() {
    let ws = tempdir().unwrap();
    write_unit(ws.path(), "checked", "grep -q magic src/data.txt");
    let marker_dir = tempdir().unwrap();
    write_mutator(ws.path(), marker_dir.path());
    let state = tempdir().unwrap();

    let out = cli(
        &["mutate", "--units"],
        &[
            ws.path().as_os_str(),
            "--state".as_ref(),
            state.path().as_os_str(),
        ],
    );
    assert!(
        out.status.success(),
        "{}",
        String::from_utf8_lossy(&out.stderr)
    );
    assert!(String::from_utf8_lossy(&out.stdout).contains("ALL STRONG"));

    // Add the vacuous unit: no longer all-strong, exit 1.
    write_unit(ws.path(), "weak", "true");
    let out = cli(
        &["mutate", "--units"],
        &[
            ws.path().as_os_str(),
            "--state".as_ref(),
            state.path().as_os_str(),
        ],
    );
    assert_eq!(out.status.code(), Some(1));
    assert!(String::from_utf8_lossy(&out.stdout).contains("NOT ALL STRONG"));
}

// Keep TempDir imports honest across cfgs.
#[allow(dead_code)]
fn _hold(_: &TempDir) {}
