//! T11 — the standalone CLI over the `array_test` library (D11: consumer-agnostic;
//! this binary is what an embedder without Rust linkage drives).

use array_test::hash::Hash;
use array_test::round::run_round;
use std::path::PathBuf;
use std::process::ExitCode;

const USAGE: &str = "\
array-test — deterministic, provable regression rounds

USAGE:
  array-test run --units <dir> --state <dir> [--round N] [--seed N] [--toolchain-hash blake3:HEX]
  array-test verify --state <dir>
  array-test tap -- <command> [args...]

run     Execute one regression round. Exit 0 iff the round is green (all cells Pass).
verify  Re-verify the ledger chain and the latest round's root. Exit 0 iff intact.
tap     Run a libtest-style command and emit deterministic, timing-free TAP on stdout
        (the evidence adapter for wrapping e.g. `cargo test` inside a cell).
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
        Some("tap") => cmd_tap(&args[1..]),
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
    // D16 precedence: explicit --toolchain-hash > <units>/toolchain.lock > sentinel.
    let toolchain = match arg_value(args, "--toolchain-hash") {
        None => None,
        Some(s) => match s.parse::<Hash>() {
            Ok(h) => Some(h),
            Err(e) => return fail(&e.to_string()),
        },
    };

    // Phase J (D7/D17): opt-in via <units>/judge.toml. The judged path owns its own
    // round loop (repair attempts are rounds), so --round is det-only.
    let units_path = PathBuf::from(&units);
    let state_path = PathBuf::from(&state);
    match array_test::judge::load_judge_config(&units_path) {
        Err(e) => {
            eprintln!("error: {e}");
            return ExitCode::from(2);
        }
        Ok(Some(config)) => {
            return cmd_run_judged(&units_path, &state_path, seed, toolchain, &config);
        }
        Ok(None) => {}
    }

    match run_round(&units_path, &state_path, round, seed, toolchain) {
        Ok(report) => {
            for cell in &report.cells {
                println!(
                    "  {:<24} [{}] {:?} ({})",
                    cell.unit_id,
                    cell.scope.as_str(),
                    cell.det_status,
                    match cell.kind {
                        array_test::round::CellOutcomeKind::Executed => "executed",
                        array_test::round::CellOutcomeKind::Reused => "reused",
                        array_test::round::CellOutcomeKind::Skipped => "skipped",
                    }
                );
            }
            println!(
                "R{}: {} cells, {} executed, {} reused, {} skipped, root {}",
                report.record.round,
                report.record.cells,
                report.executed(),
                report.reused(),
                report.skipped(),
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

fn cmd_run_judged(
    units_path: &std::path::Path,
    state_path: &std::path::Path,
    seed: u64,
    toolchain: Option<Hash>,
    config: &array_test::judge::JudgeConfig,
) -> ExitCode {
    match array_test::judge::run_with_judgment(units_path, state_path, seed, toolchain, config) {
        Ok(outcome) => {
            for cell in &outcome.judged {
                println!(
                    "  judged {:<17} [{}] {} ({}/{} runs{})",
                    cell.unit_id,
                    cell.scope.as_str(),
                    if cell.judgment.verdict { "PASS" } else { "REJECTED" },
                    cell.judgment.pass_runs,
                    cell.judgment.total_runs,
                    if cell.cached { ", cached" } else { "" }
                );
            }
            let judge_green =
                outcome.det.record.all_pass && outcome.judged.iter().all(|c| c.judgment.verdict);
            println!(
                "R{}: det {} | judge {} | {} repair attempt(s) | root {}",
                outcome.det.record.round,
                if outcome.det.record.all_pass { "green" } else { "RED" },
                if judge_green { "green" } else { "RED" },
                outcome.repair_attempts,
                outcome.det.record.root
            );
            if let Some(record) = &outcome.failure_record {
                println!("failure record: {}", record.display());
            }
            if outcome.green {
                println!("ALL PASS (two-phase)");
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

fn cmd_tap(args: &[String]) -> ExitCode {
    let command: Vec<String> = match args.iter().position(|a| a == "--") {
        Some(i) => args[i + 1..].to_vec(),
        None => return fail("tap requires -- followed by the inner command"),
    };
    match array_test::tap::run_wrapper(&command) {
        Ok((tap, success)) => {
            print!("{tap}");
            if success {
                ExitCode::SUCCESS
            } else {
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

    // Full audit (D19): confirmations chain, every root certificate, judgments chain,
    // evidence store. Problems are integrity violations; notes are informational —
    // never mixed, so success can't read as failure or vice versa.
    let report = array_test::audit::full_audit(&PathBuf::from(state));

    println!(
        "audit: {} confirmations, {} root certificate(s), {} judgment(s), {} evidence file(s)",
        report.confirmations, report.roots_checked, report.judgments, report.evidence_files
    );
    for note in &report.notes {
        println!("note: {note}");
    }
    if report.clean() {
        println!("VERIFIED");
        ExitCode::SUCCESS
    } else {
        for problem in &report.problems {
            eprintln!("PROBLEM: {problem}");
        }
        eprintln!("verification FAILED ({} problem(s))", report.problems.len());
        ExitCode::FAILURE
    }
}
