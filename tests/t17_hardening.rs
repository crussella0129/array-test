//! Acceptance checks AC70–AC73 (sprints/s9/sprint-plans/test-plan.md): the review
//! sprint's hardening — quarantine evidence, ledger-derived round numbering, optional
//! sprint field, sentinel domain separation.

#![cfg(unix)]

use array_test::audit::full_audit;
use array_test::hash::{domain, Hash};
use array_test::ledger::load_and_verify;
use array_test::manifest::load_manifest;
use array_test::round::{run_round, StatePaths};
use std::fs;
use std::path::Path;
use tempfile::tempdir;

fn write_unit(units_dir: &Path, id: &str, script: &str) {
    let dir = units_dir.join(id);
    fs::create_dir_all(dir.join("src")).unwrap();
    fs::write(dir.join("src/main.txt"), format!("content of {id}")).unwrap();
    fs::write(dir.join("contract.toml"), "[io]\n").unwrap();
    fs::write(
        dir.join("manifest.toml"),
        format!(
            "id = \"{id}\"\nsprint = 9\nversion = \"0.1.0\"\n\n\
             [test]\ncommand = [\"/bin/sh\", \"-c\", \"{script}\"]\n"
        ),
    )
    .unwrap();
}

#[test]
fn given_a_quarantined_cell_both_runs_evidence_should_be_stored() {
    let ws = tempdir().unwrap();
    write_unit(ws.path(), "flaky", "head -c 8 /dev/urandom");
    let state = tempdir().unwrap();

    run_round(ws.path(), state.path(), None, 0, None).unwrap();

    let paths = StatePaths::new(state.path());
    // Two disagreeing runs -> two distinct content-addressed evidence files.
    let stored = fs::read_dir(&paths.evidence_dir).unwrap().count();
    assert_eq!(stored, 2, "quarantine must persist BOTH transcripts");

    // The ledger's recorded (first-run) hash is among them, and the audit stays clean.
    let entries = load_and_verify(&paths.ledger_file).unwrap();
    let first_hash = entries[0].evidence_hash;
    assert!(paths
        .evidence_dir
        .join(format!("{}.evidence", first_hash.hex()))
        .exists());
    assert!(full_audit(state.path()).clean());
}

#[test]
fn given_a_lost_certificate_the_next_round_number_should_come_from_the_ledger() {
    let ws = tempdir().unwrap();
    write_unit(ws.path(), "a", "printf ok");
    let state = tempdir().unwrap();
    let paths = StatePaths::new(state.path());

    run_round(ws.path(), state.path(), None, 0, None).unwrap();
    // Simulate a crash shape: the round's entries exist but its certificate is gone.
    fs::remove_file(paths.roots_dir.join("R1.json")).unwrap();

    let r2 = run_round(ws.path(), state.path(), None, 0, None).unwrap();

    // Ledger-derived numbering: never reuse R1 (which would merge two attempts).
    assert_eq!(r2.record.round, 2);
    let entries = load_and_verify(&paths.ledger_file).unwrap();
    assert_eq!(entries.iter().filter(|e| e.round == 1).count(), 1);
    assert_eq!(entries.iter().filter(|e| e.round == 2).count(), 1);

    // And the audit notes the certificate-less round without failing (F16).
    let report = full_audit(state.path());
    assert!(report.clean());
    assert!(report.notes.iter().any(|n| n.contains("R1")));
}

#[test]
fn given_a_manifest_without_sprint_should_load() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("manifest.toml");
    fs::write(&path, "id = \"u.standalone\"\nversion = \"0.1.0\"\n").unwrap();

    let manifest = load_manifest(&path).unwrap();

    assert_eq!(manifest.sprint, 0);
}

#[test]
fn given_the_no_evidence_sentinel_it_should_never_collide_with_the_evidence_domain() {
    // Same bytes, different domains: the Skipped sentinel cannot impersonate evidence.
    assert_ne!(
        Hash::leaf(domain::NO_EVIDENCE, b"skipped"),
        Hash::leaf(domain::EVIDENCE, b"skipped")
    );
}
