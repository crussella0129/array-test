//! Acceptance checks AC24–AC28 (sprints/s3/sprint-plans/test-plan.md).

use array_test::hash::Hash;
use array_test::ledger::{array_root, load_and_verify, DetStatus, Ledger, RootRecord};
use std::fs;
use tempfile::tempdir;

fn key(n: u8) -> Hash {
    Hash::of(&[n])
}

fn ev(n: u8) -> Hash {
    Hash::of(&[0xE0, n])
}

#[test]
fn given_appended_entries_should_load_and_verify_in_order() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("confirmations.ndjson");

    let (mut ledger, existing) = Ledger::open(&path).unwrap();
    assert!(existing.is_empty());
    ledger.append(1, key(1), DetStatus::Pass, ev(1), 100).unwrap();
    ledger.append(1, key(2), DetStatus::Fail, ev(2), 101).unwrap();
    ledger.append(1, key(3), DetStatus::Pass, ev(3), 102).unwrap();

    let entries = load_and_verify(&path).unwrap();

    assert_eq!(entries.len(), 3);
    assert_eq!(entries[0].cell_key, key(1));
    assert_eq!(entries[1].det_status, DetStatus::Fail);
    assert_eq!(entries[2].seq, 2);
}

#[test]
fn given_a_reopened_ledger_should_continue_the_chain() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("confirmations.ndjson");

    {
        let (mut ledger, _) = Ledger::open(&path).unwrap();
        ledger.append(1, key(1), DetStatus::Pass, ev(1), 100).unwrap();
    }
    {
        let (mut ledger, existing) = Ledger::open(&path).unwrap();
        assert_eq!(existing.len(), 1);
        ledger.append(1, key(2), DetStatus::Pass, ev(2), 101).unwrap();
    }

    let entries = load_and_verify(&path).unwrap();
    assert_eq!(entries.len(), 2);
    assert_eq!(entries[1].prev, entries[0].entry_hash);
}

#[test]
fn given_a_tampered_status_byte_should_fail_verification() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("confirmations.ndjson");

    let (mut ledger, _) = Ledger::open(&path).unwrap();
    ledger.append(1, key(1), DetStatus::Fail, ev(1), 100).unwrap();

    // Flip the recorded status from fail to pass without recomputing hashes.
    let text = fs::read_to_string(&path).unwrap();
    let tampered = text.replace("\"fail\"", "\"pass\"");
    assert_ne!(text, tampered, "test setup: replacement must apply");
    fs::write(&path, tampered).unwrap();

    assert!(load_and_verify(&path).is_err());
}

#[test]
fn given_a_truncated_ledger_head_should_fail_verification() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("confirmations.ndjson");

    let (mut ledger, _) = Ledger::open(&path).unwrap();
    ledger.append(1, key(1), DetStatus::Pass, ev(1), 100).unwrap();
    ledger.append(1, key(2), DetStatus::Pass, ev(2), 101).unwrap();

    // Drop the first line: seq/prev checks must notice the missing head.
    let text = fs::read_to_string(&path).unwrap();
    let without_head: String = text.lines().skip(1).map(|l| format!("{l}\n")).collect();
    fs::write(&path, without_head).unwrap();

    assert!(load_and_verify(&path).is_err());
}

#[test]
fn given_the_same_cell_set_should_produce_the_same_root_regardless_of_append_order() {
    let dir = tempdir().unwrap();

    let path_a = dir.path().join("a.ndjson");
    let (mut a, _) = Ledger::open(&path_a).unwrap();
    a.append(1, key(1), DetStatus::Pass, ev(1), 100).unwrap();
    a.append(1, key(2), DetStatus::Pass, ev(2), 200).unwrap();

    let path_b = dir.path().join("b.ndjson");
    let (mut b, _) = Ledger::open(&path_b).unwrap();
    b.append(1, key(2), DetStatus::Pass, ev(2), 999).unwrap();
    b.append(1, key(1), DetStatus::Pass, ev(1), 998).unwrap();

    let root_a = array_root(&load_and_verify(&path_a).unwrap());
    let root_b = array_root(&load_and_verify(&path_b).unwrap());

    // Different order, different timestamps — same {cell_key -> status} — same root.
    assert_eq!(root_a, root_b);
}

#[test]
fn given_a_status_flip_or_added_cell_should_change_the_root() {
    let dir = tempdir().unwrap();

    let base = dir.path().join("base.ndjson");
    let (mut l, _) = Ledger::open(&base).unwrap();
    l.append(1, key(1), DetStatus::Pass, ev(1), 1).unwrap();
    let base_root = array_root(&load_and_verify(&base).unwrap());

    let flipped = dir.path().join("flipped.ndjson");
    let (mut l, _) = Ledger::open(&flipped).unwrap();
    l.append(1, key(1), DetStatus::Fail, ev(1), 1).unwrap();
    let flipped_root = array_root(&load_and_verify(&flipped).unwrap());

    let grown = dir.path().join("grown.ndjson");
    let (mut l, _) = Ledger::open(&grown).unwrap();
    l.append(1, key(1), DetStatus::Pass, ev(1), 1).unwrap();
    l.append(1, key(2), DetStatus::Pass, ev(2), 2).unwrap();
    let grown_root = array_root(&load_and_verify(&grown).unwrap());

    assert_ne!(base_root, flipped_root);
    assert_ne!(base_root, grown_root);
}

#[test]
fn given_a_later_entry_for_the_same_cell_the_latest_status_should_win() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("confirmations.ndjson");

    let (mut ledger, _) = Ledger::open(&path).unwrap();
    ledger.append(1, key(1), DetStatus::Fail, ev(1), 100).unwrap();
    ledger.append(1, key(1), DetStatus::Pass, ev(2), 200).unwrap();

    let entries = load_and_verify(&path).unwrap();
    let record = RootRecord::from_entries(1, &entries);

    assert_eq!(record.cells, 1);
    assert!(record.all_pass);
}

#[test]
fn given_quarantined_or_timed_out_cells_should_be_visible_and_not_green() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("confirmations.ndjson");

    let (mut ledger, _) = Ledger::open(&path).unwrap();
    ledger.append(1, key(1), DetStatus::Pass, ev(1), 1).unwrap();
    ledger.append(1, key(2), DetStatus::Quarantined, ev(2), 2).unwrap();
    ledger.append(1, key(3), DetStatus::TimedOut, ev(3), 3).unwrap();

    let entries = load_and_verify(&path).unwrap();
    let statuses: Vec<DetStatus> = entries.iter().map(|e| e.det_status).collect();
    assert!(statuses.contains(&DetStatus::Quarantined));
    assert!(statuses.contains(&DetStatus::TimedOut));

    let record = RootRecord::from_entries(1, &entries);
    assert!(!record.all_pass);
}

#[test]
fn given_an_empty_cell_set_should_not_be_green() {
    let record = RootRecord::from_entries(1, &[]);
    assert!(!record.all_pass);
    assert_eq!(record.cells, 0);
}

#[test]
fn given_a_root_record_should_round_trip_through_the_roots_file() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("confirmations.ndjson");

    let (mut ledger, _) = Ledger::open(&path).unwrap();
    ledger.append(7, key(1), DetStatus::Pass, ev(1), 1).unwrap();

    let entries = load_and_verify(&path).unwrap();
    let record = RootRecord::from_entries(7, &entries);
    let roots_dir = dir.path().join("roots");
    let written = record.write(&roots_dir).unwrap();

    assert!(written.ends_with("R7.json"));
    assert_eq!(RootRecord::read(&written).unwrap(), record);
}
