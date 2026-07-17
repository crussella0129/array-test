//! Rollback soundness: the append-only ledger is *history*, but the array *root* is a pure
//! function of the current tree content — so when a project's tree moves backwards (a
//! `git checkout` / `git revert` / branch switch, which for the engine is just files
//! returning to earlier bytes), the root returns with it. A content-addressed state that
//! only ever appended would be useless next to version control; this proves it doesn't.
//!
//! Two floors, both asserted:
//!   1. warm — if the state dir survives the checkout, the earlier root is reproduced from
//!      cache with zero re-execution;
//!   2. cold — if the state dir did NOT survive, the earlier root is still reproduced, by
//!      deterministic re-execution. Rollback never depends on the cache.

#![cfg(unix)]

use array_test::audit::full_audit;
use array_test::round::run_round;
use std::fs;
use std::path::Path;
use tempfile::tempdir;

/// Two units: `b` depends on `a`, so a change to `a` also moves `b`'s closure cell — the
/// "backwards arrow". Rolling the change back must return BOTH to their earlier keys.
fn write_workspace(units: &Path, token: &str) {
    let a = units.join("a");
    fs::create_dir_all(a.join("src")).unwrap();
    fs::write(a.join("src/data.txt"), format!("token={token}\n")).unwrap();
    fs::write(a.join("contract.toml"), "[io]\n").unwrap();
    fs::write(
        a.join("manifest.toml"),
        "id = \"a\"\nversion = \"1.0.0\"\n\n[tests.unit]\n\
         command = [\"/bin/sh\", \"-c\", \"grep -q token src/data.txt\"]\n",
    )
    .unwrap();

    let b = units.join("b");
    fs::create_dir_all(b.join("src")).unwrap();
    fs::write(b.join("src/lib.txt"), "b body\n").unwrap();
    fs::write(b.join("contract.toml"), "[io]\n").unwrap();
    fs::write(
        b.join("manifest.toml"),
        "id = \"b\"\nversion = \"1.0.0\"\ndeps = [\"a\"]\n\n[tests.closure]\n\
         command = [\"/bin/true\"]\n",
    )
    .unwrap();
}

#[test]
fn given_a_reverted_tree_the_root_returns_from_cache_with_zero_reexecution() {
    let units = tempdir().unwrap();
    let state = tempdir().unwrap();
    write_workspace(units.path(), "A");

    let baseline = run_round(units.path(), state.path(), None, 0, None).unwrap();
    let root_a = baseline.record.root;

    // Move the tree forward (edit `a`); both `a`'s cell and `b`'s closure cell re-key.
    write_workspace(units.path(), "B");
    let changed = run_round(units.path(), state.path(), None, 0, None).unwrap();
    assert_ne!(
        changed.record.root, root_a,
        "a real change must move the root"
    );
    assert_eq!(changed.executed(), 2, "both cells re-run on the change");

    // Move the tree BACK (the git-checkout scenario, reduced to content). The state dir
    // survived, so this is served entirely from cache.
    write_workspace(units.path(), "A");
    let reverted = run_round(units.path(), state.path(), None, 0, None).unwrap();

    assert_eq!(
        reverted.record.root, root_a,
        "reverting the tree must return the exact earlier root"
    );
    assert_eq!(reverted.executed(), 0, "rollback re-executes nothing…");
    assert_eq!(reverted.reused(), 2, "…every cell is reused from cache");

    // The ledger is append-only — three rounds, latest == the reverted root — and still
    // verifies. History is immutable; HEAD tracks the tree.
    let audit = full_audit(state.path());
    assert!(audit.clean(), "problems: {:?}", audit.problems);
    assert_eq!(audit.confirmations, 6, "3 rounds x 2 cells, none rewritten");
}

#[test]
fn given_a_discarded_state_the_reverted_tree_still_reproduces_the_root() {
    let units = tempdir().unwrap();
    write_workspace(units.path(), "A");

    // Baseline root in one state dir.
    let s1 = tempdir().unwrap();
    let root_a = run_round(units.path(), s1.path(), None, 0, None)
        .unwrap()
        .record
        .root;

    // Churn forward and back in that state (so the cache holds "B", not "A").
    write_workspace(units.path(), "B");
    run_round(units.path(), s1.path(), None, 0, None).unwrap();
    write_workspace(units.path(), "A");

    // Now simulate a checkout whose state dir did NOT come along: a brand-new, empty state
    // over the reverted tree. No cache can help — yet the root is identical, because the
    // root is a deterministic function of content. This is the rollback floor.
    let s2 = tempdir().unwrap();
    let cold = run_round(units.path(), s2.path(), None, 0, None).unwrap();
    assert_eq!(cold.executed(), 2, "cold state must actually re-run");
    assert_eq!(
        cold.record.root, root_a,
        "a cold run over reverted content reproduces the earlier root"
    );
}
