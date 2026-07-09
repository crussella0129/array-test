//! Acceptance checks AC1-AC4 (sprints/s1/sprint-plans/test-plan.md).

use array_test::hash::{compute_cell_key, compute_code_hash, CellKeyInputs, Hash};
use array_test::manifest::load_manifest;
use std::fs;
use tempfile::{tempdir, TempDir};

fn make_unit(files: &[(&str, &str)], contract: &str) -> TempDir {
    let dir = tempdir().unwrap();
    fs::create_dir_all(dir.path().join("src")).unwrap();
    for (rel, content) in files {
        let p = dir.path().join("src").join(rel);
        if let Some(parent) = p.parent() {
            fs::create_dir_all(parent).unwrap();
        }
        fs::write(p, content).unwrap();
    }
    fs::write(dir.path().join("contract.toml"), contract).unwrap();
    dir
}

#[test]
fn given_byte_identical_src_and_contract_should_produce_identical_code_hash() {
    let a = make_unit(&[("lib.rs", "fn f() {}")], "[io]\ninput=\"Bytes\"\n");
    let b = make_unit(&[("lib.rs", "fn f() {}")], "[io]\ninput=\"Bytes\"\n");

    let actual = (
        compute_code_hash(a.path()).unwrap(),
        compute_code_hash(b.path()).unwrap(),
    );
    let expected_equal = true;

    assert_eq!(actual.0 == actual.1, expected_equal);
}

#[test]
fn given_a_single_byte_change_in_src_should_change_code_hash() {
    let a = make_unit(&[("lib.rs", "fn f() {}")], "[io]\ninput=\"Bytes\"\n");
    let b = make_unit(&[("lib.rs", "fn g() {}")], "[io]\ninput=\"Bytes\"\n");

    let actual = compute_code_hash(a.path()).unwrap() == compute_code_hash(b.path()).unwrap();
    let expected_equal = false;

    assert_eq!(actual, expected_equal);
}

#[test]
fn given_a_single_byte_change_in_contract_should_change_code_hash() {
    let a = make_unit(&[("lib.rs", "fn f() {}")], "[io]\ninput=\"Bytes\"\n");
    let b = make_unit(&[("lib.rs", "fn f() {}")], "[io]\ninput=\"bytes\"\n");

    let actual = compute_code_hash(a.path()).unwrap() == compute_code_hash(b.path()).unwrap();
    let expected_equal = false;

    assert_eq!(actual, expected_equal);
}

#[test]
fn given_a_file_rename_with_same_bytes_should_change_code_hash() {
    // Path is hashed alongside content, so a rename is not invisible.
    let a = make_unit(&[("a.rs", "fn f() {}")], "[io]\ninput=\"Bytes\"\n");
    let b = make_unit(&[("b.rs", "fn f() {}")], "[io]\ninput=\"Bytes\"\n");

    let actual = compute_code_hash(a.path()).unwrap() == compute_code_hash(b.path()).unwrap();
    let expected_equal = false;

    assert_eq!(actual, expected_equal);
}

fn base_cell_key_inputs(dep_hashes: &[Hash]) -> CellKeyInputs<'_> {
    CellKeyInputs {
        target_code_hash: Hash::of(b"target"),
        scope_dep_hashes_in_dag_order: dep_hashes,
        test_def_hash: Hash::of(b"test-def"),
        fixtures_hash: Hash::of(b"fixtures"),
        seed: 42,
        toolchain_hash: Hash::of(b"toolchain"),
    }
}

#[test]
fn given_target_code_hash_changes_should_change_cell_key() {
    let deps = [Hash::of(b"dep1")];
    let mut a = base_cell_key_inputs(&deps);
    let mut b = base_cell_key_inputs(&deps);
    a.target_code_hash = Hash::of(b"target-a");
    b.target_code_hash = Hash::of(b"target-b");

    assert_ne!(compute_cell_key(&a), compute_cell_key(&b));
}

#[test]
fn given_scope_dep_hashes_change_should_change_cell_key() {
    let deps_a = [Hash::of(b"dep1")];
    let deps_b = [Hash::of(b"dep2")];
    let a = base_cell_key_inputs(&deps_a);
    let b = base_cell_key_inputs(&deps_b);

    assert_ne!(compute_cell_key(&a), compute_cell_key(&b));
}

#[test]
fn given_scope_dep_order_changes_should_change_cell_key() {
    // Order is part of the key by design (ARCHITECTURE.md §2: "in DAG order") — callers
    // must supply a consistent order, or the key will spuriously change.
    let deps_a = [Hash::of(b"dep1"), Hash::of(b"dep2")];
    let deps_b = [Hash::of(b"dep2"), Hash::of(b"dep1")];
    let a = base_cell_key_inputs(&deps_a);
    let b = base_cell_key_inputs(&deps_b);

    assert_ne!(compute_cell_key(&a), compute_cell_key(&b));
}

#[test]
fn given_test_def_hash_changes_should_change_cell_key() {
    let deps = [Hash::of(b"dep1")];
    let mut a = base_cell_key_inputs(&deps);
    let mut b = base_cell_key_inputs(&deps);
    a.test_def_hash = Hash::of(b"test-a");
    b.test_def_hash = Hash::of(b"test-b");

    assert_ne!(compute_cell_key(&a), compute_cell_key(&b));
}

#[test]
fn given_fixtures_hash_changes_should_change_cell_key() {
    let deps = [Hash::of(b"dep1")];
    let mut a = base_cell_key_inputs(&deps);
    let mut b = base_cell_key_inputs(&deps);
    a.fixtures_hash = Hash::of(b"fixtures-a");
    b.fixtures_hash = Hash::of(b"fixtures-b");

    assert_ne!(compute_cell_key(&a), compute_cell_key(&b));
}

#[test]
fn given_seed_changes_should_change_cell_key() {
    let deps = [Hash::of(b"dep1")];
    let mut a = base_cell_key_inputs(&deps);
    let mut b = base_cell_key_inputs(&deps);
    a.seed = 1;
    b.seed = 2;

    assert_ne!(compute_cell_key(&a), compute_cell_key(&b));
}

#[test]
fn given_toolchain_hash_changes_should_change_cell_key() {
    let deps = [Hash::of(b"dep1")];
    let mut a = base_cell_key_inputs(&deps);
    let mut b = base_cell_key_inputs(&deps);
    a.toolchain_hash = Hash::of(b"toolchain-a");
    b.toolchain_hash = Hash::of(b"toolchain-b");

    assert_ne!(compute_cell_key(&a), compute_cell_key(&b));
}

#[test]
fn given_no_inputs_change_should_reproduce_identical_cell_key() {
    let deps = [Hash::of(b"dep1"), Hash::of(b"dep2")];
    let a = base_cell_key_inputs(&deps);
    let b = base_cell_key_inputs(&deps);

    assert_eq!(compute_cell_key(&a), compute_cell_key(&b));
}

#[test]
fn given_a_manifest_missing_a_required_field_should_be_rejected() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("manifest.toml");
    // Missing `version`.
    fs::write(&path, "id = \"u.a\"\nsprint = 1\ndeps = []\n").unwrap();

    let result = load_manifest(&path);

    assert!(result.is_err());
}

#[test]
fn given_a_manifest_with_a_malformed_field_should_be_rejected() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("manifest.toml");
    // `sprint` must be an integer, not a string.
    fs::write(
        &path,
        "id = \"u.a\"\nsprint = \"one\"\nversion = \"0.1.0\"\n",
    )
    .unwrap();

    let result = load_manifest(&path);

    assert!(result.is_err());
}

#[test]
fn given_a_well_formed_manifest_should_load_successfully() {
    let dir = tempdir().unwrap();
    let path = dir.path().join("manifest.toml");
    fs::write(
        &path,
        "id = \"u.a\"\nsprint = 1\nversion = \"0.1.0\"\ndeps = [\"u.b\"]\n",
    )
    .unwrap();

    let manifest = load_manifest(&path).unwrap();

    assert_eq!(manifest.id, "u.a");
    assert_eq!(manifest.sprint, 1);
    assert_eq!(manifest.version, "0.1.0");
    assert_eq!(manifest.deps, vec!["u.b".to_string()]);
}
