//! T13 — the fuzz tier (D24). The fuzzer is a command (fourth use of the pattern): it
//! probes a unit and, on findings, writes crashing inputs into the unit's corpus
//! (`fixtures/fuzz/`). The loop closes through content addressing, as s2 §2.6
//! predicted: findings move `fixtures_hash`, the unit's cells re-key, and the next
//! round runs the tests against the grown corpus — no coupling beyond the filesystem.
//!
//! Sidecar throughout (D20): own contexts, own chained `fuzz.ndjson`, own cache. Clean
//! results memoize under `(code_hash, fuzzer_hash, fixtures_hash)` — honest because
//! the fuzzer contract requires seed-determinism within its budget (a nondeterministic
//! fuzzer only wastes its own cache).

use crate::hash::{compute_fixtures_hash, domain, Hash};
use crate::round::{load_workspace, RoundError, StatePaths};
use serde::{Deserialize, Serialize};
use std::fs;
use std::path::{Path, PathBuf};
use std::process::Command;
use thiserror::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct FuzzConfig {
    /// Fuzzer command; receives ARRAY_TEST_UNIT_DIR, ARRAY_TEST_CORPUS_DIR
    /// (`<unit>/fixtures/fuzz/`, created if absent), ARRAY_TEST_SEED,
    /// ARRAY_TEST_FUZZ_BUDGET. Exit 0 = clean; exit 65 = findings written to corpus.
    pub command: Vec<String>,
    #[serde(default = "default_budget_secs")]
    pub budget_secs: u64,
}

fn default_budget_secs() -> u64 {
    30
}

/// Fuzzer exit code meaning "findings written into the corpus".
pub const FINDINGS_EXIT: i32 = 65;

#[derive(Debug, Error)]
pub enum FuzzError {
    #[error("failed to read fuzz config at {path}: {source}")]
    ConfigIo {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse fuzz config at {path}: {source}")]
    ConfigParse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("fuzz config: {0}")]
    ConfigInvalid(String),
    // F4: distinguish spawn / malformed-line / chain-broken from generic I/O.
    #[error("failed to spawn fuzzer {program:?}: {source}")]
    Spawn {
        program: String,
        #[source]
        source: std::io::Error,
    },
    #[error("malformed fuzz line {line}: {reason}")]
    Malformed { line: usize, reason: String },
    #[error("fuzz chain broken at seq {seq}: {reason}")]
    ChainBroken { seq: u64, reason: String },
    #[error("fuzzer failed on unit '{unit}': exit {code:?}")]
    FuzzerFailed { unit: String, code: Option<i32> },
    #[error("io error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Round(#[from] RoundError),
    #[error(transparent)]
    CodeHash(#[from] crate::hash::CodeHashError),
}

pub fn load_fuzz_config(units_dir: &Path) -> Result<Option<FuzzConfig>, FuzzError> {
    let path = units_dir.join("fuzz.toml");
    if !path.exists() {
        return Ok(None);
    }
    let text = fs::read_to_string(&path).map_err(|source| FuzzError::ConfigIo {
        path: path.clone(),
        source,
    })?;
    let config: FuzzConfig =
        toml::from_str(&text).map_err(|source| FuzzError::ConfigParse { path, source })?;
    if config.command.is_empty() {
        return Err(FuzzError::ConfigInvalid("command must be non-empty".into()));
    }
    Ok(Some(config))
}

pub fn fuzzer_hash(config: &FuzzConfig) -> Hash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(config.command.len() as u64).to_le_bytes());
    for part in &config.command {
        bytes.extend_from_slice(&(part.len() as u64).to_le_bytes());
        bytes.extend_from_slice(part.as_bytes());
    }
    bytes.extend_from_slice(&config.budget_secs.to_le_bytes());
    Hash::leaf(domain::FUZZER, &bytes)
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzOutcomeRecord {
    pub unit_id: String,
    pub code_hash: Hash,
    pub fuzzer_hash: Hash,
    pub fixtures_before: Hash,
    pub fixtures_after: Hash,
    pub findings: bool,
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct FuzzEntry {
    pub seq: u64,
    #[serde(flatten)]
    pub outcome: FuzzOutcomeRecord,
    pub ts: u64,
    pub prev: Hash,
    pub entry_hash: Hash,
}

fn fuzz_genesis() -> Hash {
    Hash::leaf(domain::FUZZ_GENESIS, b"")
}

fn fuzz_canonical(seq: u64, o: &FuzzOutcomeRecord, ts: u64, prev: &Hash) -> Vec<u8> {
    let mut out = Vec::new();
    out.extend_from_slice(&seq.to_le_bytes());
    out.extend_from_slice(&(o.unit_id.len() as u64).to_le_bytes());
    out.extend_from_slice(o.unit_id.as_bytes());
    out.extend_from_slice(o.code_hash.as_bytes());
    out.extend_from_slice(o.fuzzer_hash.as_bytes());
    out.extend_from_slice(o.fixtures_before.as_bytes());
    out.extend_from_slice(o.fixtures_after.as_bytes());
    out.push(u8::from(o.findings));
    out.extend_from_slice(&ts.to_le_bytes());
    out.extend_from_slice(prev.as_bytes());
    out
}

/// Load and chain-verify the fuzz sidecar.
pub fn read_fuzz_entries(paths: &StatePaths) -> Result<Vec<FuzzEntry>, FuzzError> {
    let file = paths.fuzz_file.clone();
    if !file.exists() {
        return Ok(Vec::new());
    }
    let text = fs::read_to_string(&file).map_err(|source| FuzzError::Io {
        path: file.clone(),
        source,
    })?;
    let mut entries = Vec::new();
    let mut expected_prev = fuzz_genesis();
    for (i, line) in text.lines().filter(|l| !l.trim().is_empty()).enumerate() {
        let entry: FuzzEntry = serde_json::from_str(line).map_err(|e| FuzzError::Malformed {
            line: i + 1,
            reason: e.to_string(),
        })?;
        let recomputed = Hash::leaf(
            domain::FUZZ_ENTRY,
            &fuzz_canonical(entry.seq, &entry.outcome, entry.ts, &entry.prev),
        );
        if entry.prev != expected_prev || recomputed != entry.entry_hash {
            return Err(FuzzError::ChainBroken {
                seq: entry.seq,
                reason: "prev link or entry hash does not match".to_string(),
            });
        }
        expected_prev = entry.entry_hash;
        entries.push(entry);
    }
    Ok(entries)
}

/// Open-once appender (F1): O(1) per append, mirroring the confirmations/judgments
/// writers — the O(N²) re-read-per-append bug is gone.
struct FuzzWriter {
    path: PathBuf,
    chain: crate::chained::ChainState,
}

impl FuzzWriter {
    fn open(paths: &StatePaths) -> Result<Self, FuzzError> {
        let existing = read_fuzz_entries(paths)?;
        let last = existing.last().map(|e| (e.entry_hash, e.seq));
        Ok(FuzzWriter {
            path: paths.fuzz_file.clone(),
            chain: crate::chained::ChainState::new(last, fuzz_genesis()),
        })
    }

    fn append(&mut self, outcome: &FuzzOutcomeRecord) -> Result<(), FuzzError> {
        let (seq, prev) = self.chain.next();
        let ts = std::time::SystemTime::now()
            .duration_since(std::time::UNIX_EPOCH)
            .map(|d| d.as_secs())
            .unwrap_or(0);
        let entry_hash = Hash::leaf(domain::FUZZ_ENTRY, &fuzz_canonical(seq, outcome, ts, &prev));
        let entry = FuzzEntry {
            seq,
            outcome: outcome.clone(),
            ts,
            prev,
            entry_hash,
        };
        let line = serde_json::to_string(&entry).expect("FuzzEntry serializes");
        crate::chained::append_ndjson_line(&self.path, &line).map_err(|source| FuzzError::Io {
            path: self.path.clone(),
            source,
        })?;
        self.chain.advance(entry_hash);
        Ok(())
    }
}

#[derive(Debug)]
pub struct FuzzReport {
    pub units: Vec<FuzzUnitOutcome>,
    /// True iff no unit produced findings this run.
    pub clean: bool,
}

#[derive(Debug)]
pub struct FuzzUnitOutcome {
    pub record: FuzzOutcomeRecord,
    pub cached: bool,
}

/// Fuzz every unit that declares tests: run the fuzzer against its corpus dir; on
/// findings the corpus grows and the unit's cells re-key for the next round.
pub fn run_fuzz(
    units_dir: &Path,
    state_dir: &Path,
    seed: u64,
    config: &FuzzConfig,
) -> Result<FuzzReport, FuzzError> {
    let ws = load_workspace(units_dir)?;
    let paths = StatePaths::new(state_dir);
    let fhash = fuzzer_hash(config);
    let cache_dir = state_dir.join("fuzz-cache");
    let mut writer = FuzzWriter::open(&paths)?;

    let mut units = Vec::new();
    for (id, info) in &ws.units {
        if info.manifest.test.is_none() && info.manifest.tests.is_empty() {
            continue;
        }
        let key_file = cache_dir.join(format!(
            "{}-{}-{}.json",
            info.code_hash.hex(),
            fhash.hex(),
            info.fixtures_hash.hex()
        ));
        // Only CLEAN results are cacheable: a findings run changed the corpus, so its
        // key is already stale by construction. F6: a corrupt/unreadable cache is now
        // surfaced, not silenced by `unwrap_or_default()`.
        if let Some(record) = crate::cache::read_cache::<FuzzOutcomeRecord>(&key_file) {
            units.push(FuzzUnitOutcome {
                record,
                cached: true,
            });
            continue;
        }

        let corpus_dir = info.dir.join("fixtures").join("fuzz");
        fs::create_dir_all(&corpus_dir).map_err(|source| FuzzError::Io {
            path: corpus_dir.clone(),
            source,
        })?;
        let fixtures_before = compute_fixtures_hash(&info.dir)?;

        let program = &config.command[0];
        let status = Command::new(program)
            .args(&config.command[1..])
            .env("ARRAY_TEST_UNIT_DIR", &info.dir)
            .env("ARRAY_TEST_CORPUS_DIR", &corpus_dir)
            .env("ARRAY_TEST_SEED", seed.to_string())
            .env("ARRAY_TEST_FUZZ_BUDGET", config.budget_secs.to_string())
            .status()
            .map_err(|source| FuzzError::Spawn {
                program: program.clone(),
                source,
            })?;
        let findings = match status.code() {
            Some(0) => false,
            Some(FINDINGS_EXIT) => true,
            code => {
                return Err(FuzzError::FuzzerFailed {
                    unit: id.clone(),
                    code,
                })
            }
        };

        let fixtures_after = compute_fixtures_hash(&info.dir)?;
        let record = FuzzOutcomeRecord {
            unit_id: id.clone(),
            code_hash: info.code_hash,
            fuzzer_hash: fhash,
            fixtures_before,
            fixtures_after,
            findings,
        };
        writer.append(&record)?;
        if !findings {
            fs::create_dir_all(&cache_dir).map_err(|source| FuzzError::Io {
                path: cache_dir.clone(),
                source,
            })?;
            fs::write(
                &key_file,
                serde_json::to_string_pretty(&record).expect("record serializes"),
            )
            .map_err(|source| FuzzError::Io {
                path: key_file.clone(),
                source,
            })?;
        }
        units.push(FuzzUnitOutcome {
            record,
            cached: false,
        });
    }

    let clean = units.iter().all(|u| !u.record.findings);
    Ok(FuzzReport { units, clean })
}
