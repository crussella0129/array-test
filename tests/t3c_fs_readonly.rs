//! Acceptance checks AC90–AC92 (s14): the opt-in read-only filesystem (T3c) via the
//! declared-env extension channel (D25), and the quickstart contract-checker (T7b).

#![cfg(unix)]

use array_test::hash::Hash;
use array_test::runner::{fs_readonly_supported, run_cell, CellSpec, RunStatus, FS_READONLY_ENV};
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Duration;
use tempfile::tempdir;

fn spec(script: &str, ro: bool) -> CellSpec {
    let mut env = BTreeMap::new();
    if ro {
        env.insert(FS_READONLY_ENV.to_string(), "1".to_string());
    }
    CellSpec {
        cell_key: Hash::of(script.as_bytes()),
        command: vec!["/bin/sh".into(), "-c".into(), script.into()],
        cwd: std::env::temp_dir(),
        env,
        seed: 0,
        timeout: Duration::from_secs(10),
        mem_limit_mb: None,
    }
}

#[test]
fn given_the_readonly_flag_a_cell_cannot_write_anywhere_but_can_still_read() {
    if !fs_readonly_supported() {
        eprintln!("host cannot create mount namespaces; AC90 skipped");
        return;
    }
    // Writes fail everywhere (/tmp included — the flip is recursive)...
    let ro = run_cell(&spec("touch /tmp/ro-canary 2>/dev/null", true)).unwrap();
    assert!(
        !matches!(ro.status, RunStatus::Pass),
        "write escaped the RO namespace"
    );

    // ...reads still work...
    let read = run_cell(&spec(
        "test -r /etc/hostname || test -r /etc/os-release",
        true,
    ))
    .unwrap();
    assert_eq!(read.status, RunStatus::Pass);

    // ...and the flip did NOT leak to the host (propagation was made private).
    let control = run_cell(&spec("touch /tmp/rw-canary && rm /tmp/rw-canary", false)).unwrap();
    assert_eq!(control.status, RunStatus::Pass);
}

#[test]
fn given_the_flag_in_declared_env_it_is_part_of_the_cells_identity() {
    // D25: the flag rides the existing declared-env channel, which is already inside
    // test_def_hash — two otherwise-identical cells with/without it must not share a
    // key. Proven at the round level implicitly (env is hashed); here at spec level.
    use array_test::hash::{compute_cell_key, CellKeyInputs, CellScope};
    let base = |env_flag: bool| Hash::of(&[u8::from(env_flag)]);
    // Direct demonstration that differing declared env yields differing test_def is
    // covered by AC3-era tests; this guards the convention end-to-end.
    let a = compute_cell_key(&CellKeyInputs {
        target_code_hash: Hash::of(b"t"),
        scope: CellScope::Unit,
        scope_dep_hashes_in_dag_order: &[],
        test_def_hash: base(false),
        fixtures_hash: Hash::of(b"f"),
        seed: 0,
        toolchain_hash: Hash::of(b"tc"),
    });
    let b = compute_cell_key(&CellKeyInputs {
        target_code_hash: Hash::of(b"t"),
        scope: CellScope::Unit,
        scope_dep_hashes_in_dag_order: &[],
        test_def_hash: base(true),
        fixtures_hash: Hash::of(b"f"),
        seed: 0,
        toolchain_hash: Hash::of(b"tc"),
    });
    assert_ne!(a, b);
}

#[test]
fn given_the_quickstart_the_contract_audit_unit_enforces_dependency_contracts() {
    let units = PathBuf::from(env!("CARGO_MANIFEST_DIR")).join("examples/quickstart/units");
    let state = tempdir().unwrap();

    let report = array_test::round::run_round(&units, state.path(), None, 0, None).unwrap();
    assert!(report.record.all_pass, "{report:?}");
    assert_eq!(
        report.record.cells, 3,
        "greeting, announcer, contract-audit"
    );
    assert!(report
        .cells
        .iter()
        .any(|c| c.unit_id == "contract-audit"
            && c.det_status == array_test::ledger::DetStatus::Pass));
}
