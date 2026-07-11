//! T8b — the `proved` guarantee tier, made *live*: a unit whose test command runs a real
//! bounded model checker (CBMC, the engine Kani wraps) over the entire input space and
//! records the verdict as a `proved`-level confirmation.
//!
//! Structured like the T7 Hypothesis test (`t7_guarantees.rs`): `#[ignore]` + a self-skip
//! so a host without CBMC reports *ignored*, never falsely *passed* (D27). The privileged
//! CI job `apt-get install`s cbmc and runs it via `--ignored`. The committed example under
//! `examples/proved-cbmc/` is the unit exercised here.

#![cfg(unix)]

use array_test::ledger::{load_and_verify, DetStatus, Guarantee};
use array_test::round::{run_round, StatePaths};
use std::fs;
use std::path::Path;
use std::process::Command;
use tempfile::tempdir;

fn have_cbmc() -> bool {
    Command::new("cbmc")
        .arg("--version")
        .status()
        .map(|s| s.success())
        .unwrap_or(false)
}

/// Copy the committed `nibble-roundtrip` unit into `dst/nibble-roundtrip`, so the test can
/// run it — and, for the falsification case, tamper with a writable copy.
fn stage_unit(dst: &Path) {
    let src =
        Path::new(env!("CARGO_MANIFEST_DIR")).join("examples/proved-cbmc/units/nibble-roundtrip");
    let unit = dst.join("nibble-roundtrip");
    fs::create_dir_all(unit.join("src")).unwrap();
    for rel in [
        "manifest.toml",
        "contract.toml",
        "src/prove.c",
        "src/run-proof.sh",
    ] {
        fs::copy(src.join(rel), unit.join(rel)).unwrap();
    }
}

/// The engine-side `proved` plumbing, coverable without any prover: a `guarantee =
/// "proved"` declaration validates, records `Guarantee::Proved`, and (unlike the ignored
/// tests above) runs on every host. The prover is what makes the claim *true*; this checks
/// the engine records the *level*.
#[test]
fn given_a_proved_declaration_it_should_validate_and_record_the_proved_level() {
    let ws = tempdir().unwrap();
    let unit = ws.path().join("declared");
    fs::create_dir_all(unit.join("src")).unwrap();
    fs::write(unit.join("src/x.txt"), "x").unwrap();
    fs::write(unit.join("contract.toml"), "[io]\n").unwrap();
    fs::write(
        unit.join("manifest.toml"),
        "id = \"declared\"\nversion = \"1.0.0\"\n\n[tests.unit]\n\
         command = [\"/bin/sh\", \"-c\", \"printf 'ok'\"]\nguarantee = \"proved\"\n",
    )
    .unwrap();
    let state = tempdir().unwrap();

    let report = run_round(ws.path(), state.path(), None, 0, None).unwrap();
    assert!(report.record.all_pass);
    let paths = StatePaths::new(state.path());
    let entries = load_and_verify(&paths.ledger_file).unwrap();
    assert_eq!(entries[0].guarantee, Guarantee::Proved);
}

#[test]
#[ignore = "requires cbmc (apt-get install -y cbmc); run via --ignored"]
fn given_a_real_cbmc_proof_cell_should_pass_and_record_the_proved_guarantee() {
    if !have_cbmc() {
        eprintln!("cbmc unavailable; T8b proved-tier check skipped on this host");
        return;
    }

    let ws = tempdir().unwrap();
    stage_unit(ws.path());
    let state = tempdir().unwrap();

    let report = run_round(ws.path(), state.path(), None, 0, None).unwrap();

    // The proof passed BOTH runs of the determinism meta-check (CBMC is deterministic),
    // and the confirmation is recorded at the `proved` level.
    assert!(
        report.record.all_pass,
        "proved cell did not pass: {report:?}"
    );
    let paths = StatePaths::new(state.path());
    let entries = load_and_verify(&paths.ledger_file).unwrap();
    assert_eq!(entries[0].guarantee, Guarantee::Proved);
    assert_eq!(entries[0].det_status, DetStatus::Pass);
}

#[test]
#[ignore = "requires cbmc (apt-get install -y cbmc); run via --ignored"]
fn given_a_false_proof_the_model_checker_should_make_the_round_red() {
    if !have_cbmc() {
        eprintln!("cbmc unavailable; T8b falsification check skipped on this host");
        return;
    }

    let ws = tempdir().unwrap();
    stage_unit(ws.path());
    // Break the harness: assert a property CBMC can refute (nibbles are never > 15, so
    // `lo_nibble(b) == 15` fails for most bytes). If the round still went green, the
    // "proof" would be proving nothing.
    let prove_c = ws.path().join("nibble-roundtrip/src/prove.c");
    let text = fs::read_to_string(&prove_c).unwrap();
    let broken = text.replace(
        "assert(combine(hi_nibble(b), lo_nibble(b)) == b);",
        "assert(lo_nibble(b) == 15);",
    );
    assert_ne!(text, broken, "expected to tamper with the harness");
    fs::write(&prove_c, broken).unwrap();
    let state = tempdir().unwrap();

    let report = run_round(ws.path(), state.path(), None, 0, None).unwrap();

    assert!(
        !report.record.all_pass,
        "a refutable assertion must make the proved cell red"
    );
}
