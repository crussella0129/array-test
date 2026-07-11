//! Acceptance checks AC52–AC55 (sprints/s6/sprint-plans/test-plan.md): memory caps,
//! network isolation (capability-dependent), ledger-recorded isolation level, and
//! toolchain.lock re-keying.

#![cfg(unix)]

use array_test::hash::Hash;
use array_test::ledger::load_and_verify;
use array_test::round::{run_round, StatePaths};
use array_test::runner::{isolation_level, run_cell, CellSpec, IsolationLevel, RunStatus};
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;
use std::time::Duration;
use tempfile::tempdir;

fn sh_spec(script: &str, mem_limit_mb: Option<u64>) -> CellSpec {
    CellSpec {
        cell_key: Hash::of(script.as_bytes()),
        command: vec!["/bin/sh".into(), "-c".into(), script.into()],
        cwd: std::env::temp_dir(),
        env: BTreeMap::new(),
        seed: 0,
        timeout: Duration::from_secs(20),
        mem_limit_mb,
    }
}

#[test]
fn given_a_cell_exceeding_its_memory_cap_should_fail_not_pass() {
    // Command substitution forces the shell to buffer ~100MB in memory (`tail -c` won't
    // do: it keeps a ring buffer of only the requested suffix). RLIMIT_AS of 32MB
    // forbids the allocation.
    let script = "v=$(head -c 100000000 /dev/zero | tr '\\0' a); printf %s \"${#v}\"";
    let capped = run_cell(&sh_spec(script, Some(32))).unwrap();
    assert!(
        !matches!(capped.status, RunStatus::Pass),
        "memory cap was not enforced: {:?}",
        capped.status
    );

    // Control: without the cap the same cell passes, so the cap is what failed it.
    let uncapped = run_cell(&sh_spec(script, None)).unwrap();
    assert_eq!(uncapped.status, RunStatus::Pass);
    assert_eq!(
        String::from_utf8_lossy(&uncapped.evidence.stdout),
        "100000000"
    );
}

#[test]
fn given_net_isolation_a_cell_should_see_only_loopback() {
    if isolation_level() != IsolationLevel::NetIsolated {
        eprintln!("host cannot create network namespaces; AC53 skipped (EnvOnly level)");
        return;
    }
    // /proc/net/dev has 2 header lines; a fresh netns contains only lo.
    let outcome = run_cell(&sh_spec("awk 'NR>2{c++} END{print c}' /proc/net/dev", None)).unwrap();
    assert_eq!(outcome.status, RunStatus::Pass);
    assert_eq!(
        String::from_utf8_lossy(&outcome.evidence.stdout).trim(),
        "1"
    );
}

fn write_green_unit(units_dir: &Path, id: &str) {
    let dir = units_dir.join(id);
    fs::create_dir_all(dir.join("src")).unwrap();
    // Distinct content per unit: code_hash covers src+contract only (not the manifest),
    // so byte-identical units would share a cell key and dedup through the cache —
    // correct behavior (identical content is identical work), wrong fixture for
    // counting executions.
    fs::write(dir.join("src/main.txt"), format!("content of {id}")).unwrap();
    fs::write(dir.join("contract.toml"), "[io]\n").unwrap();
    fs::write(
        dir.join("manifest.toml"),
        format!(
            "id = \"{id}\"\nsprint = 6\nversion = \"0.1.0\"\n\n\
             [test]\ncommand = [\"/bin/sh\", \"-c\", \"printf ok\"]\n"
        ),
    )
    .unwrap();
}

#[test]
fn given_a_round_every_ledger_entry_should_record_the_applied_isolation_level() {
    let ws = tempdir().unwrap();
    write_green_unit(ws.path(), "a");
    let state = tempdir().unwrap();

    run_round(ws.path(), state.path(), None, 0, None).unwrap();

    let paths = StatePaths::new(state.path());
    let entries = load_and_verify(&paths.ledger_file).unwrap();
    assert!(!entries.is_empty());
    for entry in &entries {
        assert_eq!(entry.isolation, isolation_level());
    }
}

#[test]
fn given_a_toolchain_lock_change_every_cell_should_rekey_and_explicit_hash_overrides() {
    let ws = tempdir().unwrap();
    write_green_unit(ws.path(), "a");
    write_green_unit(ws.path(), "b");
    let state = tempdir().unwrap();

    let r1 = run_round(ws.path(), state.path(), None, 0, None).unwrap();
    assert_eq!(r1.executed(), 2);

    // Writing toolchain.lock re-keys everything.
    fs::write(ws.path().join("toolchain.lock"), "rustc 1.94.1 (e408947bf)").unwrap();
    let r2 = run_round(ws.path(), state.path(), None, 0, None).unwrap();
    assert_eq!(r2.executed(), 2);
    assert_ne!(r1.record.root, r2.record.root);

    // Changing its content re-keys again.
    fs::write(ws.path().join("toolchain.lock"), "rustc 1.95.0 (aaaaaaaaa)").unwrap();
    let r3 = run_round(ws.path(), state.path(), None, 0, None).unwrap();
    assert_eq!(r3.executed(), 2);

    // An explicit hash overrides the lock: matching r1's default-sentinel keys means
    // the lock file was ignored — everything reuses and the root matches r1.
    let explicit = array_test::round::unpinned_toolchain();
    let r4 = run_round(ws.path(), state.path(), None, 0, Some(explicit)).unwrap();
    assert_eq!(r4.executed(), 0);
    assert_eq!(r4.reused(), 2);
    assert_eq!(r4.record.root, r1.record.root);
}
