//! T5 — the regression round orchestrator: `R_k` as a function (ARCHITECTURE.md §3).
//!
//! v1 semantics (D13):
//! - One CLOSURE-scope cell per unit that declares `[test]`. The cell key includes the
//!   transitive dependency closure's `code_hash`es in topological order, so the
//!   schema's "backwards" arrow is *emergent*: changing a dependency changes every
//!   transitive dependent's key, putting exactly the impact set into the frontier.
//! - Cache: `Pass` and `Fail` are both reusable forever per key; `Quarantined` and
//!   `TimedOut` never enter the cache.
//! - The round root commits to the round's planned cells only — stale keys from
//!   earlier rounds stay in history but never leak into the current certificate.

use crate::dag::{Dag, DagError};
use crate::hash::{
    compute_cell_key, compute_code_hash, domain, CellKeyInputs, CodeHashError, Hash,
};
use crate::ledger::{DetStatus, Ledger, LedgerEntry, LedgerError, RootRecord};
use crate::manifest::{load_manifest, Manifest, ManifestError};
use crate::runner::{run_cell_checked, CellSpec, RunError, RunStatus, Verdict};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

pub const DEFAULT_TIMEOUT_SECS: u64 = 60;

/// The honest "nobody pinned the toolchain" sentinel (gap R-h, s4 research §2.5).
pub fn unpinned_toolchain() -> Hash {
    Hash::of(b"array-test/v1/toolchain-unpinned")
}

#[derive(Debug, Error)]
pub enum RoundError {
    #[error("io error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error(transparent)]
    Manifest(#[from] ManifestError),
    #[error(transparent)]
    CodeHash(#[from] CodeHashError),
    #[error(transparent)]
    Dag(#[from] DagError),
    #[error(transparent)]
    Ledger(#[from] LedgerError),
    #[error(transparent)]
    Run(#[from] RunError),
    #[error("duplicate unit id '{0}'")]
    DuplicateUnitId(String),
}

#[derive(Debug)]
pub struct UnitInfo {
    pub manifest: Manifest,
    pub code_hash: Hash,
    pub dir: PathBuf,
}

#[derive(Debug)]
pub struct Workspace {
    pub units: BTreeMap<String, UnitInfo>,
    pub dag: Dag,
}

/// Load every unit directory under `units_dir` (any subdirectory containing a
/// `manifest.toml`). Unit identity is the manifest `id`; duplicates are rejected.
pub fn load_workspace(units_dir: &Path) -> Result<Workspace, RoundError> {
    let mut units: BTreeMap<String, UnitInfo> = BTreeMap::new();
    let entries = fs::read_dir(units_dir).map_err(|source| RoundError::Io {
        path: units_dir.to_path_buf(),
        source,
    })?;
    let mut dirs: Vec<PathBuf> = entries
        .filter_map(|e| e.ok().map(|e| e.path()))
        .filter(|p| p.is_dir() && p.join("manifest.toml").is_file())
        .collect();
    dirs.sort();

    for dir in dirs {
        let manifest = load_manifest(&dir.join("manifest.toml"))?;
        let code_hash = compute_code_hash(&dir)?;
        let id = manifest.id.clone();
        if units
            .insert(
                id.clone(),
                UnitInfo {
                    manifest,
                    code_hash,
                    dir,
                },
            )
            .is_some()
        {
            return Err(RoundError::DuplicateUnitId(id));
        }
    }

    let dag = Dag::build(
        units
            .values()
            .map(|u| (u.manifest.id.as_str(), u.manifest.deps.as_slice())),
    )?;
    Ok(Workspace { units, dag })
}

/// Canonical bytes for `test_def_hash`: length-framed command parts, env pairs, and the
/// effective timeout. Changing any of them re-keys the cell — the test definition is an
/// input like any other (§2).
fn test_def_hash(spec: &crate::manifest::TestSpec) -> Hash {
    let mut bytes = Vec::new();
    bytes.extend_from_slice(&(spec.command.len() as u64).to_le_bytes());
    for part in &spec.command {
        bytes.extend_from_slice(&(part.len() as u64).to_le_bytes());
        bytes.extend_from_slice(part.as_bytes());
    }
    bytes.extend_from_slice(&(spec.env.len() as u64).to_le_bytes());
    for (k, v) in &spec.env {
        bytes.extend_from_slice(&(k.len() as u64).to_le_bytes());
        bytes.extend_from_slice(k.as_bytes());
        bytes.extend_from_slice(&(v.len() as u64).to_le_bytes());
        bytes.extend_from_slice(v.as_bytes());
    }
    bytes.extend_from_slice(
        &spec
            .timeout_secs
            .unwrap_or(DEFAULT_TIMEOUT_SECS)
            .to_le_bytes(),
    );
    Hash::leaf(domain::TEST_DEF, &bytes)
}

#[derive(Debug)]
pub struct CellPlan {
    pub unit_id: String,
    pub spec: CellSpec,
}

/// Derive the round's cells in topological order (D13: one CLOSURE-scope cell per unit
/// with a declared test).
pub fn plan_round(ws: &Workspace, seed: u64, toolchain_hash: Hash) -> Vec<CellPlan> {
    let topo = ws.dag.topo_order();
    let fixtures = Hash::leaf(domain::FIXTURES, b"");
    let mut plans = Vec::new();

    for id in &topo {
        let unit = &ws.units[id];
        let Some(test) = &unit.manifest.test else {
            continue;
        };
        let closure = ws.dag.closure(id);
        // Dep hashes in topo order — the canonical "in DAG order" for cell keys (§2).
        let dep_hashes: Vec<Hash> = topo
            .iter()
            .filter(|dep| closure.contains(*dep))
            .map(|dep| ws.units[dep].code_hash)
            .collect();

        let cell_key = compute_cell_key(&CellKeyInputs {
            target_code_hash: unit.code_hash,
            scope_dep_hashes_in_dag_order: &dep_hashes,
            test_def_hash: test_def_hash(test),
            fixtures_hash: fixtures,
            seed,
            toolchain_hash,
        });

        plans.push(CellPlan {
            unit_id: id.clone(),
            spec: CellSpec {
                cell_key,
                command: test.command.clone(),
                cwd: unit.dir.clone(),
                env: test.env.clone(),
                seed,
                timeout: Duration::from_secs(
                    test.timeout_secs.unwrap_or(DEFAULT_TIMEOUT_SECS),
                ),
            },
        });
    }
    plans
}

#[derive(Debug, Serialize, Deserialize)]
struct CachedConfirmation {
    cell_key: Hash,
    det_status: DetStatus,
    evidence_hash: Hash,
}

fn cache_path(cache_dir: &Path, cell_key: &Hash) -> PathBuf {
    cache_dir.join(format!("{}.json", cell_key.hex()))
}

fn cache_read(cache_dir: &Path, cell_key: &Hash) -> Option<CachedConfirmation> {
    let text = fs::read_to_string(cache_path(cache_dir, cell_key)).ok()?;
    let cached: CachedConfirmation = serde_json::from_str(&text).ok()?;
    (cached.cell_key == *cell_key).then_some(cached)
}

fn cache_write(cache_dir: &Path, cached: &CachedConfirmation) -> Result<(), RoundError> {
    fs::create_dir_all(cache_dir).map_err(|source| RoundError::Io {
        path: cache_dir.to_path_buf(),
        source,
    })?;
    let path = cache_path(cache_dir, &cached.cell_key);
    let json = serde_json::to_string_pretty(cached).expect("CachedConfirmation serializes");
    fs::write(&path, json).map_err(|source| RoundError::Io { path, source })
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum CellOutcomeKind {
    Executed,
    Reused,
}

#[derive(Debug)]
pub struct CellReport {
    pub unit_id: String,
    pub cell_key: Hash,
    pub det_status: DetStatus,
    pub kind: CellOutcomeKind,
}

#[derive(Debug)]
pub struct RoundReport {
    pub record: RootRecord,
    pub cells: Vec<CellReport>,
}

impl RoundReport {
    pub fn executed(&self) -> usize {
        self.cells
            .iter()
            .filter(|c| c.kind == CellOutcomeKind::Executed)
            .count()
    }
    pub fn reused(&self) -> usize {
        self.cells
            .iter()
            .filter(|c| c.kind == CellOutcomeKind::Reused)
            .count()
    }
}

/// State-directory layout (ARCHITECTURE.md §8) rooted at `state_dir`:
/// `ledger/confirmations.ndjson`, `ledger/roots/R<k>.json`, `cache/<cell_key>.json`.
pub struct StatePaths {
    pub ledger_file: PathBuf,
    pub roots_dir: PathBuf,
    pub cache_dir: PathBuf,
}

impl StatePaths {
    pub fn new(state_dir: &Path) -> StatePaths {
        StatePaths {
            ledger_file: state_dir.join("ledger").join("confirmations.ndjson"),
            roots_dir: state_dir.join("ledger").join("roots"),
            cache_dir: state_dir.join("cache"),
        }
    }

    /// Next round number: one past the highest existing `R<k>.json`, starting at 1.
    pub fn next_round(&self) -> u32 {
        let Ok(entries) = fs::read_dir(&self.roots_dir) else {
            return 1;
        };
        entries
            .filter_map(|e| e.ok())
            .filter_map(|e| {
                let name = e.file_name().into_string().ok()?;
                name.strip_prefix('R')?
                    .strip_suffix(".json")?
                    .parse::<u32>()
                    .ok()
            })
            .max()
            .map(|max| max + 1)
            .unwrap_or(1)
    }
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Execute one regression round: plan cells, reuse every confirmation whose key is
/// cached, hermetically run the rest, append everything to the ledger (reused entries
/// marked), and certify the round root.
pub fn run_round(
    units_dir: &Path,
    state_dir: &Path,
    round: Option<u32>,
    seed: u64,
    toolchain_hash: Hash,
) -> Result<RoundReport, RoundError> {
    let ws = load_workspace(units_dir)?;
    let plans = plan_round(&ws, seed, toolchain_hash);

    let paths = StatePaths::new(state_dir);
    fs::create_dir_all(paths.ledger_file.parent().unwrap()).map_err(|source| {
        RoundError::Io {
            path: paths.ledger_file.clone(),
            source,
        }
    })?;
    let round = round.unwrap_or_else(|| paths.next_round());
    let (mut ledger, _history) = Ledger::open(&paths.ledger_file)?;

    let mut round_entries: Vec<LedgerEntry> = Vec::new();
    let mut cells: Vec<CellReport> = Vec::new();

    for plan in &plans {
        let ts = now_unix_secs();
        let (det_status, evidence_hash, kind) =
            match cache_read(&paths.cache_dir, &plan.spec.cell_key) {
                Some(cached) => (
                    cached.det_status,
                    cached.evidence_hash,
                    CellOutcomeKind::Reused,
                ),
                None => {
                    let verdict = run_cell_checked(&plan.spec)?;
                    let (status, evidence_hash) = match verdict {
                        Verdict::Confirmed(outcome) => {
                            let status = match outcome.status {
                                RunStatus::Pass => DetStatus::Pass,
                                RunStatus::Fail { .. } => DetStatus::Fail,
                                RunStatus::TimedOut => DetStatus::TimedOut,
                            };
                            (status, outcome.evidence_hash)
                        }
                        Verdict::Quarantined { first, .. } => (DetStatus::Quarantined, first),
                    };
                    // D13: only reproducible verdicts enter the cache.
                    if matches!(status, DetStatus::Pass | DetStatus::Fail) {
                        cache_write(
                            &paths.cache_dir,
                            &CachedConfirmation {
                                cell_key: plan.spec.cell_key,
                                det_status: status,
                                evidence_hash,
                            },
                        )?;
                    }
                    (status, evidence_hash, CellOutcomeKind::Executed)
                }
            };

        let entry = ledger.append_entry(
            round,
            plan.spec.cell_key,
            det_status,
            evidence_hash,
            ts,
            kind == CellOutcomeKind::Reused,
        )?;
        round_entries.push(entry);
        cells.push(CellReport {
            unit_id: plan.unit_id.clone(),
            cell_key: plan.spec.cell_key,
            det_status,
            kind,
        });
    }

    // D13: the certificate speaks for the planned cells of THIS round only.
    let record = RootRecord::from_entries(round, &round_entries);
    record.write(&paths.roots_dir)?;

    Ok(RoundReport { record, cells })
}
