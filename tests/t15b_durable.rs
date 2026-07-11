//! Acceptance checks AC75–AC76 (sprints/s10/sprint-plans/test-plan.md): the rot guard
//! over the committed durable ledger — the artifact whose existence froze the v1 hash
//! contexts (D21). Pure file verification: no execution, fully machine-independent.

use array_test::audit::full_audit;
use array_test::ledger::{load_and_verify, RootRecord};
use array_test::round::StatePaths;
use std::path::PathBuf;

fn committed_state() -> PathBuf {
    PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("selfhost/state")
}

#[test]
fn given_the_committed_founding_ledger_the_full_audit_should_stay_clean_forever() {
    let report = full_audit(&committed_state());
    assert!(
        report.clean(),
        "the committed founding ledger no longer verifies — history was edited: {:?}",
        report.problems
    );
    assert!(report.confirmations >= 6, "R1+R2 over three cells");
    assert!(report.roots_checked >= 2);
    assert!(report.evidence_files >= 3);
}

#[test]
fn given_the_founding_rounds_r2_should_be_all_reused_history_with_an_identical_root() {
    let paths = StatePaths::new(&committed_state());

    let r1 = RootRecord::read(&paths.roots_dir.join("R1.json")).unwrap();
    let r2 = RootRecord::read(&paths.roots_dir.join("R2.json")).unwrap();
    assert!(r1.all_pass && r2.all_pass);
    assert_eq!(r1.root, r2.root, "cached round must reproduce the root");
    assert_eq!(r1.cells, 3);

    let entries = load_and_verify(&paths.ledger_file).unwrap();
    assert!(entries.iter().filter(|e| e.round == 1).all(|e| !e.reused));
    assert!(entries.iter().filter(|e| e.round == 2).all(|e| e.reused));
}
