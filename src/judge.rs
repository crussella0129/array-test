//! T9/T10 — Phase J: the judge gate and the repair micro-loop (ARCHITECTURE.md §4,
//! D7/D17/D18).
//!
//! The judge is a command (everything in this system is): it reads a unit and its
//! evidence, writes a critique to stdout whose LAST line is `rating: <0-100>`, and runs
//! `runs` times against a `threshold`. It runs unhermetically — an LLM judge needs
//! network — which is exactly why D7 keeps Phase J out of the deterministic root:
//! verdicts are recorded in their own hash-chained `judgments.ndjson` (audited, not
//! rooted, §7.3) and cached by `(cell_key, judge_hash)` — the same content-addressed
//! economics as confirmations.
//!
//! The repair micro-loop needs almost no machinery (D18): a rejected unit is patched by
//! the repair command, and the next attempt is simply another det round — the changed
//! unit re-keys, the frontier re-runs exactly what changed, Phase J re-judges only
//! moved keys. Attempts are ordinary numbered rounds in history.

use crate::hash::{domain, CellScope, Hash};
use crate::ledger::DetStatus;
use crate::round::{run_round, RoundError, RoundReport, StatePaths};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct JudgeConfig {
    /// Judge command; receives ARRAY_TEST_UNIT_DIR/_UNIT_ID/_SCOPE/_EVIDENCE/_CONTRACT.
    pub command: Vec<String>,
    /// Independent judge passes per cell (riteway ai's N-runs model).
    #[serde(default = "default_runs")]
    pub runs: u32,
    /// Percent of runs that must clear `min_rating` for a judged-pass.
    #[serde(default = "default_threshold")]
    pub threshold: u32,
    /// A run passes if its rating >= this (0–100).
    #[serde(default = "default_min_rating")]
    pub min_rating: u32,
    #[serde(default)]
    pub repair: Option<RepairConfig>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct RepairConfig {
    /// Repair command; receives ARRAY_TEST_UNIT_DIR + ARRAY_TEST_CRITIQUE, edits the
    /// unit in place.
    pub command: Vec<String>,
    /// Maximum repair attempts before escalation (§4.3).
    #[serde(default = "default_budget")]
    pub budget: u32,
}

fn default_runs() -> u32 {
    3
}
fn default_threshold() -> u32 {
    100
}
fn default_min_rating() -> u32 {
    80
}
fn default_budget() -> u32 {
    2
}

#[derive(Debug, Error)]
pub enum JudgeError {
    #[error("failed to read judge config at {path}: {source}")]
    ConfigIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse judge config at {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("judge config: {0}")]
    ConfigInvalid(String),
    #[error("failed to run judge command {program:?}: {source}")]
    Spawn {
        program: String,
        #[source]
        source: std::io::Error,
    },
    #[error("judge output has no final 'rating: <0-100>' line (cell {cell_key})")]
    NoRating { cell_key: Hash },
    #[error("io error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("critique path {reference:?} escapes the state dir")]
    UnsafePath { reference: String },
    #[error(transparent)]
    Round(#[from] RoundError),
}

/// Resolve an engine-recorded, state-relative reference (e.g. a `critique_ref`) against
/// `state_dir`, refusing anything that could escape it (F16). The references we write are
/// fixed-shape (`ledger/critiques/<64-hex>/N.md`) and cannot traverse, so this never
/// rejects a value the engine produced — it is defense-in-depth for the day a judgment is
/// loaded from disk (hence attacker-influenceable) rather than computed in-process.
fn safe_state_path(state_dir: &Path, reference: &str) -> Result<PathBuf, JudgeError> {
    let rel = Path::new(reference);
    let escapes = rel.is_absolute()
        || rel
            .components()
            .any(|c| !matches!(c, std::path::Component::Normal(_)));
    if escapes {
        return Err(JudgeError::UnsafePath {
            reference: reference.to_string(),
        });
    }
    Ok(state_dir.join(rel))
}

/// Load `<units-dir>/judge.toml` if present. Absent file = Phase J disabled.
pub fn load_judge_config(units_dir: &Path) -> Result<Option<JudgeConfig>, JudgeError> {
    let path = units_dir.join("judge.toml");
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).map_err(|source| JudgeError::ConfigIo {
        path: path.clone(),
        source,
    })?;
    let config: JudgeConfig =
        toml::from_str(&text).map_err(|source| JudgeError::ConfigParse { path, source })?;
    if config.command.is_empty() {
        return Err(JudgeError::ConfigInvalid(
            "command must be non-empty".into(),
        ));
    }
    if config.runs == 0 {
        return Err(JudgeError::ConfigInvalid("runs must be >= 1".into()));
    }
    if config.threshold > 100 || config.min_rating > 100 {
        return Err(JudgeError::ConfigInvalid(
            "threshold and min_rating are percentages (0-100)".into(),
        ));
    }
    if let Some(repair) = &config.repair {
        if repair.command.is_empty() {
            return Err(JudgeError::ConfigInvalid(
                "repair.command must be non-empty".into(),
            ));
        }
    }
    Ok(Some(config))
}

/// Pins the judge's identity: command + parameters. A changed prompt/config is a new
/// judge (R-f), so cached verdicts from the old judge are never mistaken for its.
pub fn judge_hash(config: &JudgeConfig) -> Hash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(config.command.len() as u64).to_le_bytes());
    for part in &config.command {
        bytes.extend_from_slice(&(part.len() as u64).to_le_bytes());
        bytes.extend_from_slice(part.as_bytes());
    }
    bytes.extend_from_slice(&config.runs.to_le_bytes());
    bytes.extend_from_slice(&config.threshold.to_le_bytes());
    bytes.extend_from_slice(&config.min_rating.to_le_bytes());
    Hash::leaf(domain::JUDGE, &bytes)
}

/// One judged cell's outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct CellJudgment {
    pub cell_key: Hash,
    pub judge_hash: Hash,
    pub pass_runs: u32,
    pub total_runs: u32,
    pub verdict: bool,
    /// Relative path (under the state dir) to the first critique transcript.
    pub critique_ref: String,
}

/// Hash-chained judgment ledger entry (§7.3): audited, never part of the det root.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct JudgmentEntry {
    pub seq: u64,
    pub round: u32,
    #[serde(flatten)]
    pub judgment: CellJudgment,
    pub ts: u64,
    pub prev: Hash,
    pub entry_hash: Hash,
}

fn judgment_genesis() -> Hash {
    Hash::leaf(domain::JUDGMENT_GENESIS, b"")
}

fn judgment_canonical(seq: u64, round: u32, j: &CellJudgment, ts: u64, prev: &Hash) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&seq.to_le_bytes());
    out.extend_from_slice(&round.to_le_bytes());
    out.extend_from_slice(j.cell_key.as_bytes());
    out.extend_from_slice(j.judge_hash.as_bytes());
    out.extend_from_slice(&j.pass_runs.to_le_bytes());
    out.extend_from_slice(&j.total_runs.to_le_bytes());
    out.push(u8::from(j.verdict));
    out.extend_from_slice(&(j.critique_ref.len() as u64).to_le_bytes());
    out.extend_from_slice(j.critique_ref.as_bytes());
    out.extend_from_slice(&ts.to_le_bytes());
    out.extend_from_slice(prev.as_bytes());
    out
}

/// Open-once appender for the judgments chain (F12): one verify-read at open, O(1)
/// appends thereafter — mirroring `Ledger`.
struct JudgmentWriter {
    path: PathBuf,
    last_hash: Hash,
    next_seq: u64,
}

impl JudgmentWriter {
    fn open(paths: &StatePaths) -> Result<JudgmentWriter, JudgeError> {
        let existing = read_judgments(paths)?;
        let (last_hash, next_seq) = existing
            .last()
            .map(|e| (e.entry_hash, e.seq + 1))
            .unwrap_or((judgment_genesis(), 0));
        Ok(JudgmentWriter {
            path: paths.judgments_file.clone(),
            last_hash,
            next_seq,
        })
    }

    fn append(&mut self, round: u32, judgment: &CellJudgment) -> Result<(), JudgeError> {
        use std::io::Write;
        let seq = self.next_seq;
        let prev = self.last_hash;
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let entry_hash = Hash::leaf(
            domain::JUDGMENT_ENTRY,
            &judgment_canonical(seq, round, judgment, ts, &prev),
        );
        let entry = JudgmentEntry {
            seq,
            round,
            judgment: judgment.clone(),
            ts,
            prev,
            entry_hash,
        };
        fs::create_dir_all(self.path.parent().unwrap()).map_err(|source| JudgeError::Io {
            path: self.path.clone(),
            source,
        })?;
        let mut file = fs::OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|source| JudgeError::Io {
                path: self.path.clone(),
                source,
            })?;
        writeln!(
            file,
            "{}",
            serde_json::to_string(&entry).expect("JudgmentEntry serializes")
        )
        .map_err(|source| JudgeError::Io {
            path: self.path.clone(),
            source,
        })?;
        self.last_hash = entry_hash;
        self.next_seq += 1;
        Ok(())
    }
}

/// Load and chain-verify the judgment ledger.
pub fn read_judgments(paths: &StatePaths) -> Result<Vec<JudgmentEntry>, JudgeError> {
    if !paths.judgments_file.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&paths.judgments_file).map_err(|source| JudgeError::Io {
        path: paths.judgments_file.clone(),
        source,
    })?;
    let mut entries = Vec::new();
    let mut expected_prev = judgment_genesis();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let entry: JudgmentEntry = serde_json::from_str(line)
            .map_err(|e| JudgeError::ConfigInvalid(format!("malformed judgment line: {e}")))?;
        let recomputed = Hash::leaf(
            domain::JUDGMENT_ENTRY,
            &judgment_canonical(
                entry.seq,
                entry.round,
                &entry.judgment,
                entry.ts,
                &entry.prev,
            ),
        );
        if entry.prev != expected_prev || recomputed != entry.entry_hash {
            return Err(JudgeError::ConfigInvalid(format!(
                "judgment chain broken at seq {}",
                entry.seq
            )));
        }
        expected_prev = entry.entry_hash;
        entries.push(entry);
    }
    Ok(entries)
}

fn judge_cache_path(paths: &StatePaths, cell_key: &Hash, judge: &Hash) -> PathBuf {
    paths
        .judge_cache_dir
        .join(format!("{}-{}.json", cell_key.hex(), judge.hex()))
}

fn parse_rating(stdout: &str) -> Option<u32> {
    stdout
        .lines()
        .rev()
        .find(|l| !l.trim().is_empty())?
        .trim()
        .strip_prefix("rating:")
        .and_then(|r| r.trim().parse::<u32>().ok())
        .filter(|r| *r <= 100)
}

/// The cell as the judge sees it.
struct CellUnderJudgment<'a> {
    unit_dir: &'a Path,
    unit_id: &'a str,
    scope: CellScope,
    cell_key: Hash,
    evidence_hash: Hash,
}

/// Judge one det-Pass cell: `runs` independent passes, verdict by threshold. Critique
/// transcripts land under `ledger/critiques/<cell_key>/`.
fn judge_cell(
    config: &JudgeConfig,
    jhash: Hash,
    paths: &StatePaths,
    cell: &CellUnderJudgment,
) -> Result<CellJudgment, JudgeError> {
    let critique_dir = paths.critiques_dir.join(cell.cell_key.hex());
    fs::create_dir_all(&critique_dir).map_err(|source| JudgeError::Io {
        path: critique_dir.clone(),
        source,
    })?;
    let evidence_path = paths
        .evidence_dir
        .join(format!("{}.evidence", cell.evidence_hash.hex()));

    let mut pass_runs = 0u32;
    let mut first_ref = String::new();
    for n in 0..config.runs {
        let program = &config.command[0];
        let output = Command::new(program)
            .args(&config.command[1..])
            .env("ARRAY_TEST_UNIT_DIR", cell.unit_dir)
            .env("ARRAY_TEST_UNIT_ID", cell.unit_id)
            .env("ARRAY_TEST_SCOPE", cell.scope.as_str())
            .env("ARRAY_TEST_EVIDENCE", &evidence_path)
            .env("ARRAY_TEST_CONTRACT", cell.unit_dir.join("contract.toml"))
            .output()
            .map_err(|source| JudgeError::Spawn {
                program: program.clone(),
                source,
            })?;
        let stdout = String::from_utf8_lossy(&output.stdout).into_owned();
        let rating = parse_rating(&stdout).ok_or(JudgeError::NoRating {
            cell_key: cell.cell_key,
        })?;

        let critique_path = critique_dir.join(format!("{n}.md"));
        fs::write(&critique_path, &stdout).map_err(|source| JudgeError::Io {
            path: critique_path.clone(),
            source,
        })?;
        if n == 0 {
            first_ref = format!("ledger/critiques/{}/0.md", cell.cell_key.hex());
        }
        if rating >= config.min_rating {
            pass_runs += 1;
        }
    }

    let verdict = pass_runs * 100 >= config.threshold * config.runs;
    Ok(CellJudgment {
        cell_key: cell.cell_key,
        judge_hash: jhash,
        pass_runs,
        total_runs: config.runs,
        verdict,
        critique_ref: first_ref,
    })
}

#[derive(Debug)]
pub struct JudgedCell {
    pub unit_id: String,
    pub scope: CellScope,
    pub judgment: CellJudgment,
    pub cached: bool,
}

#[derive(Debug)]
pub struct JudgedRun {
    /// The final det round (the last attempt).
    pub det: RoundReport,
    /// Judgments over the final round's det-Pass cells.
    pub judged: Vec<JudgedCell>,
    /// Repair attempts consumed (0 = judged green first try, or no repair configured).
    pub repair_attempts: u32,
    /// det green AND every judged verdict positive.
    pub green: bool,
    /// Set when a judged-red outcome was escalated to a failure record.
    pub failure_record: Option<PathBuf>,
}

/// Phase J over one det round (§4.2: only entered for det-Pass cells, only meaningful
/// on a det-green round). Verdicts are cached by `(cell_key, judge_hash)`.
fn judge_round(
    config: &JudgeConfig,
    paths: &StatePaths,
    ws_units: &BTreeMapUnitDirs,
    det: &RoundReport,
) -> Result<Vec<JudgedCell>, JudgeError> {
    let jhash = judge_hash(config);
    let mut writer = JudgmentWriter::open(paths)?;
    let mut judged = Vec::new();
    for cell in &det.cells {
        if cell.det_status != DetStatus::Pass {
            continue;
        }
        let cache_path = judge_cache_path(paths, &cell.cell_key, &jhash);
        {
            if let Some(cached) = crate::cache::read_cache::<CellJudgment>(&cache_path) {
                judged.push(JudgedCell {
                    unit_id: cell.unit_id.clone(),
                    scope: cell.scope,
                    judgment: cached,
                    cached: true,
                });
                continue;
            }
        }
        let judgment = judge_cell(
            config,
            jhash,
            paths,
            &CellUnderJudgment {
                unit_dir: &ws_units[&cell.unit_id],
                unit_id: &cell.unit_id,
                scope: cell.scope,
                cell_key: cell.cell_key,
                evidence_hash: cell.evidence_hash,
            },
        )?;
        writer.append(det.record.round, &judgment)?;
        fs::create_dir_all(&paths.judge_cache_dir).map_err(|source| JudgeError::Io {
            path: paths.judge_cache_dir.clone(),
            source,
        })?;
        fs::write(
            &cache_path,
            serde_json::to_string_pretty(&judgment).expect("CellJudgment serializes"),
        )
        .map_err(|source| JudgeError::Io {
            path: cache_path,
            source,
        })?;
        judged.push(JudgedCell {
            unit_id: cell.unit_id.clone(),
            scope: cell.scope,
            judgment,
            cached: false,
        });
    }
    Ok(judged)
}

type BTreeMapUnitDirs = std::collections::BTreeMap<String, PathBuf>;

fn unit_dirs(units_dir: &Path) -> Result<BTreeMapUnitDirs, JudgeError> {
    let ws = crate::round::load_workspace(units_dir)?;
    Ok(ws
        .units
        .into_iter()
        .map(|(id, info)| (id, info.dir))
        .collect())
}

fn write_failure_record(
    paths: &StatePaths,
    round: u32,
    rejected: &[&JudgedCell],
) -> Result<PathBuf, JudgeError> {
    fs::create_dir_all(&paths.failures_dir).map_err(|source| JudgeError::Io {
        path: paths.failures_dir.clone(),
        source,
    })?;
    let path = paths.failures_dir.join(format!("R{round}-judgment.md"));
    let mut body = format!(
        "# Judgment failure — R{round}\n\nDet phase was green; Phase J rejected {} cell(s).\n\n",
        rejected.len()
    );
    for cell in rejected {
        body.push_str(&format!(
            "- `{}` [{}] — {}/{} runs passed; critique: `{}`\n",
            cell.unit_id,
            cell.scope.as_str(),
            cell.judgment.pass_runs,
            cell.judgment.total_runs,
            cell.judgment.critique_ref
        ));
    }
    fs::write(&path, body).map_err(|source| JudgeError::Io {
        path: path.clone(),
        source,
    })?;
    Ok(path)
}

/// Run the operator-declared repair command against a single rejected unit.
///
/// # Safety / trust boundary
/// This spawns `repair.command` — an operator-authored program from `judge.toml`, at the
/// same trust level as the test commands themselves. `unit_dir` is an engine-enumerated
/// workspace path; `critique` has already passed [`safe_state_path`] and so is guaranteed
/// to live under the state dir. Nothing attacker-controlled reaches the argv or the two
/// exported env vars. The child is *not* sandboxed here (repair edits the unit in place by
/// design); the following det round re-runs its tests under the normal sandbox.
fn run_repair(repair: &RepairConfig, unit_dir: &Path, critique: &Path) -> Result<(), JudgeError> {
    let program = &repair.command[0];
    Command::new(program)
        .args(&repair.command[1..])
        .env("ARRAY_TEST_UNIT_DIR", unit_dir)
        .env("ARRAY_TEST_CRITIQUE", critique)
        .status()
        .map_err(|source| JudgeError::Spawn {
            program: program.clone(),
            source,
        })?;
    Ok(())
}

/// The full two-phase gate with the repair micro-loop (§4, D18): det round → judge →
/// (on rejection, repair the unit and run ANOTHER det round — the frontier machinery
/// re-runs exactly what the repair changed) → … until green or budget exhausted.
pub fn run_with_judgment(
    units_dir: &Path,
    state_dir: &Path,
    seed: u64,
    toolchain_hash: Option<Hash>,
    config: &JudgeConfig,
) -> Result<JudgedRun, JudgeError> {
    let paths = StatePaths::new(state_dir);
    let budget = config.repair.as_ref().map(|r| r.budget).unwrap_or(0);
    let mut attempts = 0u32;

    loop {
        let det = run_round(units_dir, state_dir, None, seed, toolchain_hash)?;
        if !det.record.all_pass {
            // Phase D failure: handled by the existing machinery (§4.1); Phase J is
            // never entered and repair does not apply.
            return Ok(JudgedRun {
                det,
                judged: Vec::new(),
                repair_attempts: attempts,
                green: false,
                failure_record: None,
            });
        }

        let dirs = unit_dirs(units_dir)?;
        let judged = judge_round(config, &paths, &dirs, &det)?;
        let rejected: Vec<&JudgedCell> = judged.iter().filter(|c| !c.judgment.verdict).collect();

        if rejected.is_empty() {
            return Ok(JudgedRun {
                det,
                judged,
                repair_attempts: attempts,
                green: true,
                failure_record: None,
            });
        }

        if attempts >= budget {
            let record = write_failure_record(&paths, det.record.round, &rejected)?;
            return Ok(JudgedRun {
                det,
                judged,
                repair_attempts: attempts,
                green: false,
                failure_record: Some(record),
            });
        }

        // §4.3: repair each rejected unit, scoped to that unit, driven by its critique.
        let repair = config.repair.as_ref().expect("budget > 0 implies repair");
        for cell in &rejected {
            // F16: `critique_ref` is engine-generated (`ledger/critiques/<cell_key>/N.md`,
            // a 64-hex path component), so it cannot traverse — but validate before
            // joining anyway, as defense-in-depth against a future path that feeds a
            // disk-loaded (hence attacker-influenceable) judgment here.
            let critique = safe_state_path(state_dir, &cell.judgment.critique_ref)?;
            run_repair(repair, &dirs[&cell.unit_id], &critique)?;
        }
        attempts += 1;
        // Loop: the next det round re-keys whatever the repair touched.
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn safe_state_path_accepts_engine_shaped_refs_and_rejects_traversal() {
        let state = Path::new("/state");
        // The shape the engine actually writes.
        let ok = safe_state_path(state, "ledger/critiques/abcd/0.md").unwrap();
        assert_eq!(ok, Path::new("/state/ledger/critiques/abcd/0.md"));

        for bad in [
            "../escape",
            "a/../../etc/passwd",
            "/etc/passwd",
            "./x/../..",
        ] {
            assert!(
                matches!(
                    safe_state_path(state, bad),
                    Err(JudgeError::UnsafePath { .. })
                ),
                "ref {bad:?} should have been refused"
            );
        }
    }
}
