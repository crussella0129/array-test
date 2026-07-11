//! T6 — the TAP evidence adapter (D14).
//!
//! Principle: determinism is produced **at the source**, never by normalizing evidence
//! after the fact. This module gives a cell command (`array-test tap -- <cmd…>`) whose
//! stdout is minimal, sorted, timing-free TAP — so the runner keeps hashing exactly
//! what the cell emitted, byte for byte, and the determinism meta-check keeps its full
//! power. The adapter is part of the test definition (inside `test_def_hash`), not part
//! of the trust boundary.

use std::process::Command;
use thiserror::Error;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum PointStatus {
    Ok,
    Failed,
    Ignored,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TestPoint {
    pub name: String,
    pub status: PointStatus,
}

/// Extract test points from libtest-style output. Only lines of the shape
/// `test <name> ... ok|FAILED|ignored[…]` survive; timings, `running N tests`
/// headers, cargo chatter, and everything else is dropped.
pub fn parse_libtest(output: &str) -> Vec<TestPoint> {
    let mut points = Vec::new();
    for line in output.lines() {
        let Some(rest) = line.strip_prefix("test ") else {
            continue;
        };
        let Some((name, verdict)) = rest.rsplit_once(" ... ") else {
            continue;
        };
        let status = if verdict == "ok" {
            PointStatus::Ok
        } else if verdict == "FAILED" {
            PointStatus::Failed
        } else if verdict == "ignored" || verdict.starts_with("ignored,") {
            PointStatus::Ignored
        } else {
            continue;
        };
        points.push(TestPoint {
            name: name.to_string(),
            status,
        });
    }
    points
}

/// Render sorted, minimal TAP 13. `inner_succeeded` is the wrapped command's exit
/// success; if it failed without any parsed FAILED point, a synthetic `not ok` is
/// appended — silence must never read as success.
pub fn render_tap(mut points: Vec<TestPoint>, inner_succeeded: bool) -> String {
    points.sort_by(|a, b| a.name.cmp(&b.name));

    let any_failed = points.iter().any(|p| p.status == PointStatus::Failed);
    let synthesize = !inner_succeeded && !any_failed;
    let total = points.len() + usize::from(synthesize);

    let mut out = String::from("TAP version 13\n");
    out.push_str(&format!("1..{total}\n"));
    for (i, point) in points.iter().enumerate() {
        let n = i + 1;
        match point.status {
            PointStatus::Ok => out.push_str(&format!("ok {n} - {}\n", point.name)),
            PointStatus::Failed => out.push_str(&format!("not ok {n} - {}\n", point.name)),
            PointStatus::Ignored => out.push_str(&format!("ok {n} - {} # SKIP\n", point.name)),
        }
    }
    if synthesize {
        out.push_str(&format!("not ok {total} - inner process exited nonzero\n"));
    }
    out
}

#[derive(Debug, Error)]
pub enum TapError {
    #[error("tap wrapper needs a command after --")]
    EmptyCommand,
    #[error("failed to run {program:?}: {source}")]
    Spawn {
        program: String,
        #[source]
        source: std::io::Error,
    },
}

/// Run the inner command, returning `(tap_output, success)`. The wrapper runs *inside*
/// an already-hermetic cell, so the child inherits the cell's environment untouched;
/// both inner streams are consumed (cargo chatter on stderr dies here), and only the
/// rendered TAP goes to the wrapper's stdout.
pub fn run_wrapper(command: &[String]) -> Result<(String, bool), TapError> {
    let program = command.first().ok_or(TapError::EmptyCommand)?;
    let output = Command::new(program)
        .args(&command[1..])
        .output()
        .map_err(|source| TapError::Spawn {
            program: program.clone(),
            source,
        })?;

    let stdout = String::from_utf8_lossy(&output.stdout);
    let points = parse_libtest(&stdout);
    let inner_succeeded = output.status.success();
    let all_points_ok = !points.iter().any(|p| p.status == PointStatus::Failed);

    let tap = render_tap(points, inner_succeeded);
    Ok((tap, inner_succeeded && all_points_ok))
}

#[cfg(test)]
mod tests {
    use super::*;

    const NOISY: &str = "\
   Compiling array-test v0.1.0
    Finished `test` profile [unoptimized] target(s) in 0.53s
     Running tests/x.rs (target/debug/deps/x-abc123)

running 3 tests
test zeta_last ... ok
test alpha_first ... ok
test middle_one ... ignored, not yet implemented

test result: ok. 2 passed; 0 failed; 1 ignored; finished in 0.02s
";

    #[test]
    fn given_noisy_libtest_output_should_emit_only_sorted_test_points() {
        let tap = render_tap(parse_libtest(NOISY), true);
        assert_eq!(
            tap,
            "TAP version 13\n\
             1..3\n\
             ok 1 - alpha_first\n\
             ok 2 - middle_one # SKIP\n\
             ok 3 - zeta_last\n"
        );
        assert!(!tap.contains("finished in"));
        assert!(!tap.contains("running"));
    }

    #[test]
    fn given_output_differing_only_in_timing_noise_should_render_identically() {
        let other = NOISY.replace("0.53s", "1.87s").replace("0.02s", "0.41s");
        assert_ne!(NOISY, other);
        assert_eq!(
            render_tap(parse_libtest(NOISY), true),
            render_tap(parse_libtest(&other), true)
        );
    }

    #[test]
    fn given_a_failed_test_should_emit_not_ok() {
        let out = "test broken_thing ... FAILED\ntest fine_thing ... ok\n";
        let tap = render_tap(parse_libtest(out), false);
        assert!(tap.contains("not ok 1 - broken_thing"));
        assert!(tap.contains("ok 2 - fine_thing"));
    }

    #[test]
    fn given_a_nonzero_inner_exit_with_no_parsed_failure_should_synthesize_not_ok() {
        let tap = render_tap(parse_libtest("test ok_one ... ok\n"), false);
        assert!(tap.contains("1..2"));
        assert!(tap.contains("not ok 2 - inner process exited nonzero"));
    }
}
