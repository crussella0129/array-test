//! T11 — the standalone CLI over the `array_test` library (D11: consumer-agnostic;
//! this binary is what an embedder without Rust linkage drives).

use array_test::hash::Hash;
use array_test::ledger::{array_root, load_and_verify, RootRecord};
use array_test::round::{run_round, unpinned_toolchain, StatePaths};
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
array-test — deterministic, provable regression rounds

USAGE:
  array-test run --units <dir> --state <dir> [--round N] [--seed N] [--toolchain-hash blake3:HEX]
  array-test verify --state <dir>

run     Execute one regression round. Exit 0 iff the round is green (all cells Pass).
verify  Re-verify the ledger chain and the latest round's root. Exit 0 iff intact.
";

fn arg_value(args: &[String], flag: &str) -> Option<String> {
    args.iter()
        .position(|a| a == flag)
        .and_then(|i| args.get(i + 1).cloned())
}

fn fail(msg: &str) -> ExitCode {
    eprintln!("error: {msg}");
    eprintln!("{USAGE}");
    ExitCode::from(2)
}

fn main() -> ExitCode {
    let args: Vec<String> = std::env::args().skip(1).collect();
    match args.first().map(String::as_str) {
        Some("run") => cmd_run(&args[1..]),
        Some("verify") => cmd_verify(&args[1..]),
        Some("--help") | Some("-h") | Some("help") => {
            println!("{USAGE}");
            ExitCode::SUCCESS
        }
        _ => fail("expected a subcommand: run | verify"),
    }
}

fn cmd_run(args: &[String]) -> ExitCode {
    let Some(units) = arg_value(args, "--units") else {
        return fail("run requires --units <dir>");
    };
    let Some(state) = arg_value(args, "--state") else {
        return fail("run requires --state <dir>");
    };
    let round = match arg_value(args, "--round").map(|v| v.parse::<u32>()) {
        None => None,
        Some(Ok(n)) => Some(n),
        Some(Err(_)) => return fail("--round must be an integer"),
    };
    let seed = match arg_value(args, "--seed").map(|v| v.parse::<u64>()) {
        None => 0,
        Some(Ok(n)) => n,
        Some(Err(_)) => return fail("--seed must be an integer"),
    };
    let toolchain = match arg_value(args, "--toolchain-hash") {
        None => unpinned_toolchain(),
        Some(s) => match s.parse::<Hash>() {
            Ok(h) => h,
            Err(e) => return fail(&e.to_string()),
        },
    };

    match run_round(
        &PathBuf::from(units),
        &PathBuf::from(state),
        round,
        seed,
        toolchain,
    ) {
        Ok(report) => {
            for cell in &report.cells {
                println!(
                    "  {:<24} {:?} ({})",
                    cell.unit_id,
                    cell.det_status,
                    match cell.kind {
                        array_test::round::CellOutcomeKind::Executed => "executed",
                        array_test::round::CellOutcomeKind::Reused => "reused",
                    }
                );
            }
            println!(
                "R{}: {} cells, {} executed, {} reused, root {}",
                report.record.round,
                report.record.cells,
                report.executed(),
                report.reused(),
                report.record.root
            );
            if report.record.all_pass {
                println!("ALL PASS");
                ExitCode::SUCCESS
            } else {
                println!("NOT GREEN");
                ExitCode::FAILURE
            }
        }
        Err(e) => {
            eprintln!("error: {e}");
            ExitCode::from(2)
        }
    }
}

fn cmd_verify(args: &[String]) -> ExitCode {
    let Some(state) = arg_value(args, "--state") else {
        return fail("verify requires --state <dir>");
    };
    let paths = StatePaths::new(&PathBuf::from(state));

    let entries = match load_and_verify(&paths.ledger_file) {
        Ok(entries) => entries,
        Err(e) => {
            eprintln!("ledger verification FAILED: {e}");
            return ExitCode::FAILURE;
        }
    };
    println!("ledger: {} entries, chain intact", entries.len());

    let latest_round = entries.iter().map(|e| e.round).max();
    let Some(round) = latest_round else {
        println!("ledger empty; nothing further to verify");
        return ExitCode::SUCCESS;
    };
    let round_entries: Vec<_> = entries
        .iter()
        .filter(|e| e.round == round)
        .cloned()
        .collect();
    let recomputed = array_root(&round_entries);

    let root_path = paths.roots_dir.join(format!("R{round}.json"));
    match RootRecord::read(&root_path) {
        Ok(record) if record.root == recomputed => {
            println!(
                "R{round}: root matches ledger ({}), all_pass={}",
                record.root, record.all_pass
            );
            ExitCode::SUCCESS
        }
        Ok(record) => {
            eprintln!(
                "root MISMATCH for R{round}: certificate {} vs ledger {}",
                record.root, recomputed
            );
            ExitCode::FAILURE
        }
        Err(e) => {
            eprintln!("cannot read root certificate {}: {e}", root_path.display());
            ExitCode::FAILURE
        }
    }
}
