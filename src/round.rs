//! T5/T5b — the regression round orchestrator: `R_k` as a function (ARCHITECTURE.md §3).
//!
//! Semantics (D13, D15):
//! - Cells come from `[tests.<scope>]` declarations (legacy `[test]` = closure). The
//!   scope decides which `code_hash`es enter the key: `unit` none, `direct` direct
//!   deps, `closure` the transitive closure, `e2e` every unit in the workspace. The
//!   schema's "backwards" arrow is *emergent*: changing a dependency changes exactly
//!   the keys whose scope covers it.
//! - Tiers run unit → direct → closure → e2e; once a completed tier holds a non-Pass,
//!   higher-tier cells are recorded `Skipped` (visible, never cached, not green).
//! - Cache: `Pass` and `Fail` are reusable forever per key; `Quarantined`, `TimedOut`,
//!   and `Skipped` never enter the cache.
//! - The round root commits to the round's planned cells only — stale keys from
//!   earlier rounds stay in history but never leak into the current certificate.
//! - Toolchain (D16): explicit hash > `<units-dir>/toolchain.lock` bytes > the honest
//!   "unpinned" sentinel.

use crate::dag::{Dag, DagError};
use crate::hash::{
    compute_cell_key, compute_code_hash, domain, CellKeyInputs, CellScope, CodeHashError, Hash,
};
use crate::ledger::{DetStatus, Guarantee, Ledger, LedgerEntry, LedgerError, RootRecord};
use crate::manifest::{load_manifest, Manifest, ManifestError, TestSpec};
use crate::runner::{run_cell_checked, CellSpec, RunError, RunStatus, Verdict};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;
use thiserror::Error;

/// Per-scope wall-clock envelope defaults (D15); `timeout_secs` overrides.
pub fn default_timeout_secs(scope: CellScope) -> u64 {
    match scope {
        CellScope::Unit => 10,
        CellScope::Direct => 30,
        CellScope::Closure => 60,
        CellScope::E2e => 300,
    }
}

/// The honest "nobody pinned the toolchain" sentinel (gap R-h, s4 research §2.5) —
/// properly domained under TOOLCHAIN (F8).
pub fn unpinned_toolchain() -> Hash {
    Hash::leaf(domain::TOOLCHAIN, b"unpinned")
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
    /// T13: content hash of `<unit>/fixtures/` (sentinel when absent). Fixtures re-key
    /// cells through their own key slot without pretending to be code.
    pub fixtures_hash: Hash,
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
        let fixtures_hash = crate::hash::compute_fixtures_hash(&dir)?;
        let id = manifest.id.clone();
        if units
            .insert(
                id.clone(),
                UnitInfo {
                    manifest,
                    code_hash,
                    fixtures_hash,
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

/// Canonical bytes for `test_def_hash`: length-framed command parts, env pairs, the
/// effective timeout, and the memory cap. Changing any of them re-keys the cell — the
/// test definition (envelope included) is an input like any other (§2).
fn test_def_hash(spec: &TestSpec, scope: CellScope) -> Hash {
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
            .unwrap_or_else(|| default_timeout_secs(scope))
            .to_le_bytes(),
    );
    bytes.extend_from_slice(&spec.mem_limit_mb.unwrap_or(0).to_le_bytes());
    bytes.push(guarantee_of(spec).byte());
    Hash::leaf(domain::TEST_DEF, &bytes)
}

/// Map the validated declaration to its enum (default `example`, §7.2).
pub fn guarantee_of(spec: &TestSpec) -> Guarantee {
    match spec.guarantee.as_deref() {
        Some("property") => Guarantee::Property,
        Some("proved") => Guarantee::Proved,
        _ => Guarantee::Example,
    }
}

#[derive(Debug)]
pub struct CellPlan {
    pub unit_id: String,
    pub scope: CellScope,
    pub guarantee: Guarantee,
    pub spec: CellSpec,
}

/// The dep hashes a scope admits into the key (D15) — this IS the scope's meaning.
fn scope_dep_hashes(ws: &Workspace, topo: &[String], id: &str, scope: CellScope) -> Vec<Hash> {
    match scope {
        CellScope::Unit => Vec::new(),
        CellScope::Direct => {
            let direct = &ws.units[id].manifest.deps;
            topo.iter()
                .filter(|u| direct.contains(u))
                .map(|u| ws.units[u].code_hash)
                .collect()
        }
        CellScope::Closure => {
            let closure = ws.dag.closure(id);
            topo.iter()
                .filter(|u| closure.contains(*u))
                .map(|u| ws.units[u].code_hash)
                .collect()
        }
        // End-to-end honestly depends on everything: every other unit's hash is in.
        CellScope::E2e => topo
            .iter()
            .filter(|u| u.as_str() != id)
            .map(|u| ws.units[u].code_hash)
            .collect(),
    }
}

/// Derive the round's cells, ordered tier-by-tier (unit → direct → closure → e2e) and
/// topologically within each tier — the fail-fast ladder's execution order (D15).
pub fn plan_round(ws: &Workspace, seed: u64, toolchain_hash: Hash) -> Vec<CellPlan> {
    let topo = ws.dag.topo_order();
    let mut plans = Vec::new();

    for scope in [
        CellScope::Unit,
        CellScope::Direct,
        CellScope::Closure,
        CellScope::E2e,
    ] {
        for id in &topo {
            let unit = &ws.units[id];
            let test = if scope == CellScope::Closure && unit.manifest.test.is_some() {
                unit.manifest.test.as_ref()
            } else {
                unit.manifest.tests.get(&scope)
            };
            let Some(test) = test else { continue };

            let dep_hashes = scope_dep_hashes(ws, &topo, id, scope);
            let cell_key = compute_cell_key(&CellKeyInputs {
                target_code_hash: unit.code_hash,
                scope,
                scope_dep_hashes_in_dag_order: &dep_hashes,
                test_def_hash: test_def_hash(test, scope),
                fixtures_hash: unit.fixtures_hash,
                seed,
                toolchain_hash,
            });

            plans.push(CellPlan {
                unit_id: id.clone(),
                scope,
                guarantee: guarantee_of(test),
                spec: CellSpec {
                    cell_key,
                    command: test.command.clone(),
                    cwd: unit.dir.clone(),
                    env: test.env.clone(),
                    seed,
                    timeout: Duration::from_secs(
                        test.timeout_secs
                            .unwrap_or_else(|| default_timeout_secs(scope)),
                    ),
                    mem_limit_mb: test.mem_limit_mb,
                },
            });
        }
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
    let cached: CachedConfirmation = crate::cache::read_cache(&cache_path(cache_dir, cell_key))?;
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
    Skipped,
}

#[derive(Debug)]
pub struct CellReport {
    pub unit_id: String,
    pub scope: CellScope,
    pub cell_key: Hash,
    pub det_status: DetStatus,
    pub evidence_hash: Hash,
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
    pub fn skipped(&self) -> usize {
        self.cells
            .iter()
            .filter(|c| c.kind == CellOutcomeKind::Skipped)
            .count()
    }
}

/// Resolve the toolchain hash (D16): explicit > `toolchain.lock` bytes > unpinned
/// sentinel.
pub fn resolve_toolchain(units_dir: &Path, explicit: Option<Hash>) -> Hash {
    if let Some(h) = explicit {
        return h;
    }
    match fs::read(units_dir.join("toolchain.lock")) {
        Ok(bytes) => Hash::leaf(domain::TOOLCHAIN, &bytes),
        Err(_) => unpinned_toolchain(),
    }
}

/// State-directory layout (ARCHITECTURE.md §8) rooted at `state_dir`:
/// `ledger/confirmations.ndjson`, `ledger/roots/R<k>.json`, `cache/<cell_key>.json`.
pub struct StatePaths {
    pub ledger_file: PathBuf,
    pub roots_dir: PathBuf,
    pub cache_dir: PathBuf,
    /// Content-addressed evidence store: `<evidence_hash>.tap` (s7 research §2.5).
    pub evidence_dir: PathBuf,
    pub judgments_file: PathBuf,
    pub critiques_dir: PathBuf,
    pub judge_cache_dir: PathBuf,
    pub failures_dir: PathBuf,
    /// T12 sidecar surfaces (added post-freeze, D20-legal).
    pub mutations_file: PathBuf,
    pub mutation_cache_dir: PathBuf,
    pub mutation_work_dir: PathBuf,
    /// T13 sidecar (added post-freeze, D20-legal).
    pub fuzz_file: PathBuf,
}

impl StatePaths {
    pub fn new(state_dir: &Path) -> StatePaths {
        StatePaths {
            ledger_file: state_dir.join("ledger").join("confirmations.ndjson"),
            roots_dir: state_dir.join("ledger").join("roots"),
            cache_dir: state_dir.join("cache"),
            evidence_dir: state_dir.join("evidence"),
            judgments_file: state_dir.join("ledger").join("judgments.ndjson"),
            critiques_dir: state_dir.join("ledger").join("critiques"),
            judge_cache_dir: state_dir.join("judge-cache"),
            failures_dir: state_dir.join("ledger").join("failures"),
            mutations_file: state_dir.join("ledger").join("mutations.ndjson"),
            mutation_cache_dir: state_dir.join("mutation-cache"),
            mutation_work_dir: state_dir.join("mutation-work"),
            fuzz_file: state_dir.join("ledger").join("fuzz.ndjson"),
        }
    }
}

/// Write an executed cell's evidence bytes under their hash (content-addressed store).
fn store_evidence(
    evidence_dir: &Path,
    outcome: &crate::runner::RunOutcome,
) -> Result<(), RoundError> {
    fs::create_dir_all(evidence_dir).map_err(|source| RoundError::Io {
        path: evidence_dir.to_path_buf(),
        source,
    })?;
    let path = evidence_dir.join(format!("{}.evidence", outcome.evidence_hash.hex()));
    if path.exists() {
        return Ok(()); // content-addressed: same hash, same bytes
    }
    // Exactly the framed bytes the hash covers — re-hashable by any verifier.
    fs::write(&path, outcome.evidence.framed()).map_err(|source| RoundError::Io { path, source })
}

fn now_unix_secs() -> u64 {
    std::time::SystemTime::now()
        .duration_since(std::time::UNIX_EPOCH)
        .map(|d| d.as_secs())
        .unwrap_or(0)
}

/// Resolve one planned cell to its `(status, evidence, outcome-kind)` — the frontier
/// economics of a single cell, lifted out of [`run_round`]'s tier loop (F3). Three paths:
/// gated (Skipped, never cached — D15), cache hit (Reused), or a hermetic run whose
/// evidence is stored and whose reproducible verdicts are cached (D13). Quarantine stores
/// both disagreeing transcripts and records the first (F9).
fn resolve_cell(
    paths: &StatePaths,
    plan: &CellPlan,
    gate_broken: bool,
    skipped_evidence: Hash,
) -> Result<(DetStatus, Hash, CellOutcomeKind), RoundError> {
    if gate_broken {
        // D15: skipping is state, not silence — and never cached.
        return Ok((
            DetStatus::Skipped,
            skipped_evidence,
            CellOutcomeKind::Skipped,
        ));
    }
    if let Some(cached) = cache_read(&paths.cache_dir, &plan.spec.cell_key) {
        return Ok((
            cached.det_status,
            cached.evidence_hash,
            CellOutcomeKind::Reused,
        ));
    }
    let (status, evidence_hash) = match run_cell_checked(&plan.spec)? {
        Verdict::Confirmed(outcome) => {
            let status = match outcome.status {
                RunStatus::Pass => DetStatus::Pass,
                RunStatus::Fail { .. } => DetStatus::Fail,
                RunStatus::TimedOut => DetStatus::TimedOut,
            };
            // Evidence store (s7 §2.5): the root must be backed by retrievable bytes,
            // not hashes of discarded data.
            store_evidence(&paths.evidence_dir, &outcome)?;
            (status, outcome.evidence_hash)
        }
        Verdict::Quarantined { first, second } => {
            // F9: quarantine means "these two disagreed" — both transcripts ARE the
            // evidence; store both, record the first's hash in the ledger as before.
            store_evidence(&paths.evidence_dir, &first)?;
            store_evidence(&paths.evidence_dir, &second)?;
            (DetStatus::Quarantined, first.evidence_hash)
        }
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
    Ok((status, evidence_hash, CellOutcomeKind::Executed))
}

/// Execute one regression round: plan cells tier-by-tier, reuse every confirmation
/// whose key is cached, hermetically run the rest, gate higher tiers behind lower-tier
/// failures (Skipped, D15), append everything to the ledger (reuse + isolation level
/// marked), and certify the round root.
pub fn run_round(
    units_dir: &Path,
    state_dir: &Path,
    round: Option<u32>,
    seed: u64,
    toolchain_hash: Option<Hash>,
) -> Result<RoundReport, RoundError> {
    let ws = load_workspace(units_dir)?;
    let toolchain = resolve_toolchain(units_dir, toolchain_hash);
    let plans = plan_round(&ws, seed, toolchain);
    let isolation = crate::runner::isolation_level();
    let skipped_evidence = Hash::leaf(domain::NO_EVIDENCE, b"skipped");

    let paths = StatePaths::new(state_dir);
    fs::create_dir_all(paths.ledger_file.parent().unwrap()).map_err(|source| RoundError::Io {
        path: paths.ledger_file.clone(),
        source,
    })?;
    let (mut ledger, history) = Ledger::open(&paths.ledger_file)?;
    // F10: the ledger is the state machine; certificates are outputs. Deriving the
    // round number from the roots dir would reuse a number after a crash between
    // ledger-append and certificate-write, merging two attempts under one round.
    let round = round.unwrap_or_else(|| {
        history
            .iter()
            .map(|e| e.round)
            .max()
            .map(|r| r + 1)
            .unwrap_or(1)
    });

    let mut round_entries: Vec<LedgerEntry> = Vec::new();
    let mut cells: Vec<CellReport> = Vec::new();
    let mut gate_broken = false; // any non-Pass in a COMPLETED lower tier
    let mut tier_failed = false; // any non-Pass in the tier in progress
    let mut current_tier: Option<CellScope> = None;

    for plan in &plans {
        if current_tier != Some(plan.scope) {
            // Tier boundary: a failed lower tier gates everything above it, but not
            // its own siblings — cells within a tier are semantically parallel.
            gate_broken = gate_broken || tier_failed;
            tier_failed = false;
            current_tier = Some(plan.scope);
        }

        let ts = now_unix_secs();
        let (det_status, evidence_hash, kind) =
            resolve_cell(&paths, plan, gate_broken, skipped_evidence)?;

        // Status gates, not freshness: a reused Fail closes higher tiers too.
        if det_status != DetStatus::Pass {
            tier_failed = true;
        }

        let entry = ledger.record(crate::ledger::ConfirmationInput {
            round,
            cell_key: plan.spec.cell_key,
            det_status,
            evidence_hash,
            ts,
            reused: kind == CellOutcomeKind::Reused,
            isolation,
            guarantee: plan.guarantee,
        })?;
        round_entries.push(entry);
        cells.push(CellReport {
            unit_id: plan.unit_id.clone(),
            scope: plan.scope,
            cell_key: plan.spec.cell_key,
            det_status,
            evidence_hash,
            kind,
        });
    }

    // D13: the certificate speaks for the planned cells of THIS round only.
    let record = RootRecord::from_entries(round, &round_entries);
    record.write(&paths.roots_dir)?;

    Ok(RoundReport { record, cells })
}
