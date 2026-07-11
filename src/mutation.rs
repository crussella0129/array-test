//! T12 — the frontier-scoped mutation tier (D23): who tests the tests, made
//! incremental by content addressing. The first post-freeze extension: everything here
//! is sidecar (its own hash-chained ledger, its own contexts) and touches no frozen
//! layout (D20/D21).
//!
//! The mutator is a command (the judge/repair pattern, third time): it receives a
//! scratch COPY of a unit and an index, corrupts the copy, and the engine asks one
//! question — does a full round over the mutant workspace stay green? **Killed ⇔ the
//! round goes red**, deliberately broader than "the unit's own cells failed": a
//! dependent's closure-scope cell catching the mutant is the integration lattice
//! doing its job, and it counts.
//!
//! Scores memoize under `(code_hash, mutator_hash, baseline_root)` — the baseline
//! root IS a commitment to the whole detection surface, so any change to any test,
//! dep, seed, or toolchain re-mutates exactly when it should, and nothing else does.

use crate::hash::{domain, Hash};
use crate::round::{load_workspace, resolve_toolchain, run_round, RoundError, StatePaths};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct MutationConfig {
    /// Mutator command; receives ARRAY_TEST_UNIT_DIR (a scratch copy),
    /// ARRAY_TEST_MUTANT_INDEX, ARRAY_TEST_SEED. Exit 0 = mutant produced; exit 64 =
    /// no mutant at this index (skipped).
    pub command: Vec<String>,
    #[serde(default = "default_mutants")]
    pub mutants: u32,
    /// Percent of produced mutants that must be killed for "mutation-strong".
    #[serde(default = "default_min_score")]
    pub min_score: u32,
}

fn default_mutants() -> u32 {
    4
}
fn default_min_score() -> u32 {
    100
}

/// Mutator exit code meaning "no mutant available at this index".
pub const NO_MUTANT_EXIT: i32 = 64;

#[derive(Debug, Error)]
pub enum MutationError {
    #[error("failed to read mutation config at {path}: {source}")]
    ConfigIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse mutation config at {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("mutation config: {0}")]
    ConfigInvalid(String),
    #[error("baseline round is not green; mutation scores would be meaningless")]
    BaselineRed,
    #[error("mutator command failed on unit '{unit}' index {index}: exit {code:?}")]
    MutatorFailed {
        unit: String,
        index: u32,
        code: Option<i32>,
    },
    #[error("io error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("symlink in workspace at {path}: cannot copy for mutation")]
    Symlink { path: PathBuf },
    #[error(transparent)]
    Round(#[from] RoundError),
}

/// Load `<units-dir>/mutation.toml` if present.
pub fn load_mutation_config(units_dir: &Path) -> Result<Option<MutationConfig>, MutationError> {
    let path = units_dir.join("mutation.toml");
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).map_err(|source| MutationError::ConfigIo {
        path: path.clone(),
        source,
    })?;
    let config: MutationConfig =
        toml::from_str(&text).map_err(|source| MutationError::ConfigParse { path, source })?;
    if config.command.is_empty() {
        return Err(MutationError::ConfigInvalid(
            "command must be non-empty".into(),
        ));
    }
    if config.mutants == 0 {
        return Err(MutationError::ConfigInvalid("mutants must be >= 1".into()));
    }
    if config.min_score > 100 {
        return Err(MutationError::ConfigInvalid(
            "min_score is a percentage (0-100)".into(),
        ));
    }
    Ok(Some(config))
}

/// Pins the mutator's identity (command + parameters), like `judge_hash` (R-f logic).
pub fn mutator_hash(config: &MutationConfig) -> Hash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(config.command.len() as u64).to_le_bytes());
    for part in &config.command {
        bytes.extend_from_slice(&(part.len() as u64).to_le_bytes());
        bytes.extend_from_slice(part.as_bytes());
    }
    bytes.extend_from_slice(&config.mutants.to_le_bytes());
    bytes.extend_from_slice(&config.min_score.to_le_bytes());
    Hash::leaf(domain::MUTATOR, &bytes)
}

/// One unit's mutation outcome.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct UnitScore {
    pub unit_id: String,
    pub code_hash: Hash,
    pub mutator_hash: Hash,
    pub baseline_root: Hash,
    /// Mutants actually produced and executed (skips excluded).
    pub mutants_run: u32,
    pub killed: u32,
    pub skipped: u32,
    pub score_pct: u32,
    pub min_score: u32,
    pub strong: bool,
}

/// Hash-chained sidecar entry (`mutations.ndjson`).
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct MutationEntry {
    pub seq: u64,
    #[serde(flatten)]
    pub score: UnitScore,
    pub ts: u64,
    pub prev: Hash,
    pub entry_hash: Hash,
}

fn mutation_genesis() -> Hash {
    Hash::leaf(domain::MUTATION_GENESIS, b"")
}

fn mutation_canonical(seq: u64, s: &UnitScore, ts: u64, prev: &Hash) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&seq.to_le_bytes());
    out.extend_from_slice(&(s.unit_id.len() as u64).to_le_bytes());
    out.extend_from_slice(s.unit_id.as_bytes());
    out.extend_from_slice(s.code_hash.as_bytes());
    out.extend_from_slice(s.mutator_hash.as_bytes());
    out.extend_from_slice(s.baseline_root.as_bytes());
    out.extend_from_slice(&s.mutants_run.to_le_bytes());
    out.extend_from_slice(&s.killed.to_le_bytes());
    out.extend_from_slice(&s.skipped.to_le_bytes());
    out.extend_from_slice(&s.score_pct.to_le_bytes());
    out.extend_from_slice(&s.min_score.to_le_bytes());
    out.push(s.strong as u8);
    out.extend_from_slice(&ts.to_le_bytes());
    out.extend_from_slice(prev.as_bytes());
    out
}

/// Load and chain-verify the mutations sidecar.
pub fn read_mutations(paths: &StatePaths) -> Result<Vec<MutationEntry>, MutationError> {
    if !paths.mutations_file.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&paths.mutations_file).map_err(|source| MutationError::Io {
        path: paths.mutations_file.clone(),
        source,
    })?;
    let mut entries = Vec::new();
    let mut expected_prev = mutation_genesis();
    for line in text.lines().filter(|l| !l.trim().is_empty()) {
        let entry: MutationEntry = serde_json::from_str(line)
            .map_err(|e| MutationError::ConfigInvalid(format!("malformed mutation line: {e}")))?;
        let recomputed = Hash::leaf(
            domain::MUTATION_ENTRY,
            &mutation_canonical(entry.seq, &entry.score, entry.ts, &entry.prev),
        );
        if entry.prev != expected_prev || recomputed != entry.entry_hash {
            return Err(MutationError::ConfigInvalid(format!(
                "mutation chain broken at seq {}",
                entry.seq
            )));
        }
        expected_prev = entry.entry_hash;
        entries.push(entry);
    }
    Ok(entries)
}

fn append_mutation(paths: &StatePaths, score: &UnitScore) -> Result<(), MutationError> {
    use std::io::Write;
    let existing = read_mutations(paths)?;
    let (prev, seq) = existing
        .last()
        .map(|e| (e.entry_hash, e.seq + 1))
        .unwrap_or((mutation_genesis(), 0));
    let ts = std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0);
    let entry_hash = Hash::leaf(
        domain::MUTATION_ENTRY,
        &mutation_canonical(seq, score, ts, &prev),
    );
    let entry = MutationEntry {
        seq,
        score: score.clone(),
        ts,
        prev,
        entry_hash,
    };
    fs::create_dir_all(paths.mutations_file.parent().unwrap()).map_err(|source| {
        MutationError::Io {
            path: paths.mutations_file.clone(),
            source,
        }
    })?;
    let mut file = fs::OpenOptions::new()
        .create(true)
        .append(true)
        .open(&paths.mutations_file)
        .map_err(|source| MutationError::Io {
            path: paths.mutations_file.clone(),
            source,
        })?;
    writeln!(
        file,
        "{}",
        serde_json::to_string(&entry).expect("MutationEntry serializes")
    )
    .map_err(|source| MutationError::Io {
        path: paths.mutations_file.clone(),
        source,
    })
}

/// Recursive copy that refuses symlinks (same doctrine as `code_hash` hashing: a link
/// out of the tree would smuggle foreign bytes into the mutant workspace).
fn copy_dir(src: &Path, dst: &Path) -> Result<(), MutationError> {
    fs::create_dir_all(dst).map_err(|source| MutationError::Io {
        path: dst.to_path_buf(),
        source,
    })?;
    let entries = fs::read_dir(src).map_err(|source| MutationError::Io {
        path: src.to_path_buf(),
        source,
    })?;
    for entry in entries {
        let entry = entry.map_err(|source| MutationError::Io {
            path: src.to_path_buf(),
            source,
        })?;
        let path = entry.path();
        let file_type = entry.file_type().map_err(|source| MutationError::Io {
            path: path.clone(),
            source,
        })?;
        if file_type.is_symlink() {
            return Err(MutationError::Symlink { path });
        }
        let target = dst.join(entry.file_name());
        if file_type.is_dir() {
            copy_dir(&path, &target)?;
        } else {
            fs::copy(&path, &target).map_err(|source| MutationError::Io {
                path: path.clone(),
                source,
            })?;
        }
    }
    Ok(())
}

fn cache_path(paths: &StatePaths, score_key: &(Hash, Hash, Hash)) -> PathBuf {
    paths.mutation_cache_dir.join(format!(
        "{}-{}-{}.json",
        score_key.0.hex(),
        score_key.1.hex(),
        score_key.2.hex()
    ))
}

#[derive(Debug)]
pub struct UnitOutcome {
    pub score: UnitScore,
    pub cached: bool,
}

#[derive(Debug)]
pub struct MutationReport {
    pub baseline_root: Hash,
    pub units: Vec<UnitOutcome>,
    pub all_strong: bool,
}

/// Run the mutation tier: green baseline, then per dirty unit (cache misses only)
/// generate mutants and score kills. Scratch rounds share one state under
/// `mutation-work/`, so unrelated cells hit cache after the first mutant — the
/// frontier economics apply inside the mutation run too.
pub fn run_mutation(
    units_dir: &Path,
    state_dir: &Path,
    seed: u64,
    toolchain: Option<Hash>,
    config: &MutationConfig,
) -> Result<MutationReport, MutationError> {
    let baseline = run_round(units_dir, state_dir, None, seed, toolchain)?;
    if !baseline.record.all_pass {
        return Err(MutationError::BaselineRed);
    }
    let baseline_root = baseline.record.root;
    let resolved_toolchain = Some(resolve_toolchain(units_dir, toolchain));

    let paths = StatePaths::new(state_dir);
    let ws = load_workspace(units_dir)?;
    let mhash = mutator_hash(config);
    let scratch_state = paths.mutation_work_dir.join("state");

    let mut units = Vec::new();
    for (id, info) in &ws.units {
        // Mutate units that declare their own tests; others have no committed claim
        // of coverage to measure.
        if info.manifest.test.is_none() && info.manifest.tests.is_empty() {
            continue;
        }
        let key = (info.code_hash, mhash, baseline_root);
        let cache_file = cache_path(&paths, &key);
        if let Ok(text) = fs::read_to_string(&cache_file) {
            if let Ok(score) = serde_json::from_str::<UnitScore>(&text) {
                units.push(UnitOutcome {
                    score,
                    cached: true,
                });
                continue;
            }
        }

        let unit_dir_name = info
            .dir
            .file_name()
            .expect("unit dir has a name")
            .to_owned();
        let mut killed = 0u32;
        let mut skipped = 0u32;
        let mut ran = 0u32;

        for index in 0..config.mutants {
            let work_units = paths
                .mutation_work_dir
                .join(format!("{}-{index}", id.replace('/', "_")));
            let _ = fs::remove_dir_all(&work_units);
            copy_dir(units_dir, &work_units)?;
            let mutant_unit_dir = work_units.join(&unit_dir_name);

            let program = &config.command[0];
            let status = Command::new(program)
                .args(&config.command[1..])
                .env("ARRAY_TEST_UNIT_DIR", &mutant_unit_dir)
                .env("ARRAY_TEST_MUTANT_INDEX", index.to_string())
                .env("ARRAY_TEST_SEED", seed.to_string())
                .status()
                .map_err(|source| MutationError::Io {
                    path: PathBuf::from(program),
                    source,
                })?;
            match status.code() {
                Some(0) => {}
                Some(NO_MUTANT_EXIT) => {
                    skipped += 1;
                    let _ = fs::remove_dir_all(&work_units);
                    continue;
                }
                code => {
                    return Err(MutationError::MutatorFailed {
                        unit: id.clone(),
                        index,
                        code,
                    });
                }
            }

            // A mutant that didn't change the unit isn't a mutant.
            if crate::hash::compute_code_hash(&mutant_unit_dir)
                .map(|h| h == info.code_hash)
                .unwrap_or(false)
            {
                skipped += 1;
                let _ = fs::remove_dir_all(&work_units);
                continue;
            }

            let round = run_round(&work_units, &scratch_state, None, seed, resolved_toolchain)?;
            if !round.record.all_pass {
                killed += 1;
            }
            ran += 1;
            let _ = fs::remove_dir_all(&work_units);
        }

        let score_pct = (killed * 100).checked_div(ran).unwrap_or(0);
        let score = UnitScore {
            unit_id: id.clone(),
            code_hash: info.code_hash,
            mutator_hash: mhash,
            baseline_root,
            mutants_run: ran,
            killed,
            skipped,
            score_pct,
            min_score: config.min_score,
            strong: ran > 0 && score_pct >= config.min_score,
        };
        append_mutation(&paths, &score)?;
        fs::create_dir_all(&paths.mutation_cache_dir).map_err(|source| MutationError::Io {
            path: paths.mutation_cache_dir.clone(),
            source,
        })?;
        fs::write(
            &cache_file,
            serde_json::to_string_pretty(&score).expect("UnitScore serializes"),
        )
        .map_err(|source| MutationError::Io {
            path: cache_file,
            source,
        })?;
        units.push(UnitOutcome {
            score,
            cached: false,
        });
    }

    let all_strong = !units.is_empty() && units.iter().all(|u| u.score.strong);
    Ok(MutationReport {
        baseline_root,
        units,
        all_strong,
    })
}
