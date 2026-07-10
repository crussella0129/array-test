//! T4 — the confirmation ledger and array root (ARCHITECTURE.md §7.1, §8).
//!
//! Two commitments with different jobs:
//!
//! - **The chain** (`confirmations.ndjson`): append-only history. Each entry's hash
//!   covers canonical fixed-length bytes of everything recorded — including wall-clock
//!   time, which history may honestly contain — and links to the previous entry, so any
//!   byte of the past is tamper-evident. Verification replays the file and recomputes
//!   every link; no trust in the writer required.
//! - **The array root**: commits to `{cell_key → det_status}` only (latest entry per
//!   cell), sorted by cell key. Timestamps and sequence are excluded, so the root is
//!   reproducible from a fresh re-run of the same cells — this is the hash a green
//!   sprint gate reads (§9), and what an embedding consumer (e.g. a sprint-loops Test
//!   phase) should treat as the round's certificate.

use crate::hash::{domain, Hash};
use serde::{Deserialize, Serialize};
use std::collections::BTreeMap;
use std::fs::{self, OpenOptions};
use std::io::{BufRead, BufReader, Write};
use std::path::{Path, PathBuf};
use thiserror::Error;

/// Deterministic-phase status. `Quarantined`, `TimedOut`, and `Skipped` are
/// first-class, visible statuses (D10, D15) — none of them is green. `Skipped` records
/// a cell gated off by a lower-tier failure (fail-fast ladder, D15); like quarantine,
/// skipping must be state, not silence.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(rename_all = "snake_case")]
pub enum DetStatus {
    Pass,
    Fail,
    Quarantined,
    TimedOut,
    Skipped,
}

impl DetStatus {
    fn byte(self) -> u8 {
        match self {
            DetStatus::Pass => 0,
            DetStatus::Fail => 1,
            DetStatus::Quarantined => 2,
            DetStatus::TimedOut => 3,
            DetStatus::Skipped => 4,
        }
    }
}

/// Declared guarantee level of the cell's test (§7.2, D17): what kind of claim a Pass
/// makes. Recorded, not verified — Phase J audits declarations.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize, Default)]
#[serde(rename_all = "snake_case")]
pub enum Guarantee {
    #[default]
    Example,
    Property,
    Proved,
}

impl Guarantee {
    pub fn byte(self) -> u8 {
        match self {
            Guarantee::Example => 0,
            Guarantee::Property => 1,
            Guarantee::Proved => 2,
        }
    }
}

#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct LedgerEntry {
    pub seq: u64,
    pub round: u32,
    pub cell_key: Hash,
    pub det_status: DetStatus,
    pub evidence_hash: Hash,
    /// Unix seconds. Inside the chain hash (history records when), outside the array
    /// root (reproducibility ignores when).
    pub ts: u64,
    /// True when this confirmation was inherited from the cache rather than freshly
    /// executed this round (D13). Inside the chain hash: the audit trail must say
    /// which confirmations were inherited.
    #[serde(default)]
    pub reused: bool,
    /// The isolation level the runner actually applied (D12/D16) — the hermeticity
    /// this confirmation was earned under. Inside the chain hash.
    #[serde(default = "default_isolation")]
    pub isolation: crate::runner::IsolationLevel,
    /// The declared guarantee level of the test (§7.2, D17). Inside the chain hash.
    #[serde(default)]
    pub guarantee: Guarantee,
    pub prev: Hash,
    pub entry_hash: Hash,
}

fn default_isolation() -> crate::runner::IsolationLevel {
    crate::runner::IsolationLevel::EnvOnly
}

fn isolation_byte(level: crate::runner::IsolationLevel) -> u8 {
    match level {
        crate::runner::IsolationLevel::EnvOnly => 0,
        crate::runner::IsolationLevel::NetIsolated => 1,
    }
}

/// Canonical bytes an entry hash covers: fixed-length fields in fixed order, no
/// serialization-format ambiguity.
#[allow(clippy::too_many_arguments)]
fn canonical_bytes(
    seq: u64,
    round: u32,
    cell_key: &Hash,
    det_status: DetStatus,
    evidence_hash: &Hash,
    ts: u64,
    reused: bool,
    isolation: crate::runner::IsolationLevel,
    guarantee: Guarantee,
    prev: &Hash,
) -> Vec<u8> {
    let mut out = Vec::with_capacity(8 + 4 + 32 + 1 + 32 + 8 + 1 + 1 + 1 + 32);
    out.extend_from_slice(&seq.to_le_bytes());
    out.extend_from_slice(&round.to_le_bytes());
    out.extend_from_slice(cell_key.as_bytes());
    out.push(det_status.byte());
    out.extend_from_slice(evidence_hash.as_bytes());
    out.extend_from_slice(&ts.to_le_bytes());
    out.push(reused as u8);
    out.push(isolation_byte(isolation));
    out.push(guarantee.byte());
    out.extend_from_slice(prev.as_bytes());
    out
}

fn genesis() -> Hash {
    Hash::leaf(domain::LEDGER_GENESIS, b"")
}

#[derive(Debug, Error)]
pub enum LedgerError {
    #[error("io error on {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("malformed ledger line {line}: {reason}")]
    Malformed { line: usize, reason: String },
    #[error("chain broken at seq {seq}: {reason}")]
    ChainBroken { seq: u64, reason: String },
}

/// Everything a confirmation records (F11) — see field docs on [`LedgerEntry`].
#[derive(Debug, Clone, Copy)]
pub struct ConfirmationInput {
    pub round: u32,
    pub cell_key: Hash,
    pub det_status: DetStatus,
    pub evidence_hash: Hash,
    pub ts: u64,
    pub reused: bool,
    pub isolation: crate::runner::IsolationLevel,
    pub guarantee: Guarantee,
}

/// An append-only, hash-chained ledger backed by an ndjson file.
#[derive(Debug)]
pub struct Ledger {
    path: PathBuf,
    last_hash: Hash,
    next_seq: u64,
}

impl Ledger {
    /// Open (creating if absent) and fully verify the existing chain.
    pub fn open(path: &Path) -> Result<(Self, Vec<LedgerEntry>), LedgerError> {
        let entries = if path.exists() {
            load_and_verify(path)?
        } else {
            Vec::new()
        };
        let (last_hash, next_seq) = entries
            .last()
            .map(|e| (e.entry_hash, e.seq + 1))
            .unwrap_or((genesis(), 0));
        Ok((
            Ledger {
                path: path.to_path_buf(),
                last_hash,
                next_seq,
            },
            entries,
        ))
    }

    /// Append a freshly-executed confirmation with default provenance; computes seq,
    /// prev link, and entry hash. For full provenance use [`Ledger::record`].
    pub fn append(
        &mut self,
        round: u32,
        cell_key: Hash,
        det_status: DetStatus,
        evidence_hash: Hash,
        ts: u64,
    ) -> Result<LedgerEntry, LedgerError> {
        self.record(ConfirmationInput {
            round,
            cell_key,
            det_status,
            evidence_hash,
            ts,
            reused: false,
            isolation: crate::runner::IsolationLevel::EnvOnly,
            guarantee: Guarantee::Example,
        })
    }

    /// Append one confirmation with full provenance: cache inheritance (D13), the
    /// isolation level it was earned under (D16), its declared guarantee (D17).
    pub fn record(&mut self, input: ConfirmationInput) -> Result<LedgerEntry, LedgerError> {
        let ConfirmationInput {
            round,
            cell_key,
            det_status,
            evidence_hash,
            ts,
            reused,
            isolation,
            guarantee,
        } = input;
        let seq = self.next_seq;
        let prev = self.last_hash;
        let entry_hash = Hash::leaf(
            domain::LEDGER_ENTRY,
            &canonical_bytes(
                seq,
                round,
                &cell_key,
                det_status,
                &evidence_hash,
                ts,
                reused,
                isolation,
                guarantee,
                &prev,
            ),
        );
        let entry = LedgerEntry {
            seq,
            round,
            cell_key,
            det_status,
            evidence_hash,
            ts,
            reused,
            isolation,
            guarantee,
            prev,
            entry_hash,
        };

        let line = serde_json::to_string(&entry).expect("LedgerEntry always serializes");
        let mut file = OpenOptions::new()
            .create(true)
            .append(true)
            .open(&self.path)
            .map_err(|source| LedgerError::Io {
                path: self.path.clone(),
                source,
            })?;
        writeln!(file, "{line}").map_err(|source| LedgerError::Io {
            path: self.path.clone(),
            source,
        })?;

        self.last_hash = entry_hash;
        self.next_seq += 1;
        Ok(entry)
    }
}

/// Load an ndjson ledger and verify every hash and chain link. This is the "no trust in
/// the runner" path (§7.1): anyone holding the file can run it.
pub fn load_and_verify(path: &Path) -> Result<Vec<LedgerEntry>, LedgerError> {
    let file = fs::File::open(path).map_err(|source| LedgerError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let mut entries = Vec::new();
    let mut expected_prev = genesis();
    for (i, line) in BufReader::new(file).lines().enumerate() {
        let line = line.map_err(|source| LedgerError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        if line.trim().is_empty() {
            continue;
        }
        let entry: LedgerEntry =
            serde_json::from_str(&line).map_err(|e| LedgerError::Malformed {
                line: i + 1,
                reason: e.to_string(),
            })?;
        if entry.seq != entries.len() as u64 {
            return Err(LedgerError::ChainBroken {
                seq: entry.seq,
                reason: format!("expected seq {}, found {}", entries.len(), entry.seq),
            });
        }
        if entry.prev != expected_prev {
            return Err(LedgerError::ChainBroken {
                seq: entry.seq,
                reason: "prev link does not match previous entry hash".to_string(),
            });
        }
        let recomputed = Hash::leaf(
            domain::LEDGER_ENTRY,
            &canonical_bytes(
                entry.seq,
                entry.round,
                &entry.cell_key,
                entry.det_status,
                &entry.evidence_hash,
                entry.ts,
                entry.reused,
                entry.isolation,
                entry.guarantee,
                &entry.prev,
            ),
        );
        if recomputed != entry.entry_hash {
            return Err(LedgerError::ChainBroken {
                seq: entry.seq,
                reason: "entry hash does not match recorded fields".to_string(),
            });
        }
        expected_prev = entry.entry_hash;
        entries.push(entry);
    }
    Ok(entries)
}

/// The array root: a commitment to `{cell_key → det_status}` (latest entry wins per
/// cell), sorted by cell key. Reproducible from a fresh run of the same cells —
/// timestamps and history shape are deliberately excluded.
pub fn array_root(entries: &[LedgerEntry]) -> Hash {
    let mut latest: BTreeMap<Hash, DetStatus> = BTreeMap::new();
    for entry in entries {
        latest.insert(entry.cell_key, entry.det_status);
    }
    let leaves: Vec<Hash> = latest
        .iter()
        .map(|(cell_key, status)| {
            let mut bytes = Vec::with_capacity(33);
            bytes.extend_from_slice(cell_key.as_bytes());
            bytes.push(status.byte());
            Hash::leaf(domain::ROOT_LEAF, &bytes)
        })
        .collect();
    Hash::node(domain::ARRAY_ROOT, &leaves)
}

/// The round certificate (`roots/R<k>.json`) — the stable artifact an embedding
/// consumer reads. `all_pass` is the green gate: quarantined or timed-out cells are
/// not green.
#[derive(Debug, Clone, Serialize, Deserialize, PartialEq, Eq)]
pub struct RootRecord {
    pub round: u32,
    pub root: Hash,
    pub cells: u64,
    pub all_pass: bool,
}

impl RootRecord {
    pub fn from_entries(round: u32, entries: &[LedgerEntry]) -> RootRecord {
        let mut latest: BTreeMap<Hash, DetStatus> = BTreeMap::new();
        for entry in entries {
            latest.insert(entry.cell_key, entry.det_status);
        }
        RootRecord {
            round,
            root: array_root(entries),
            cells: latest.len() as u64,
            all_pass: !latest.is_empty()
                && latest.values().all(|s| *s == DetStatus::Pass),
        }
    }

    pub fn write(&self, roots_dir: &Path) -> Result<PathBuf, LedgerError> {
        fs::create_dir_all(roots_dir).map_err(|source| LedgerError::Io {
            path: roots_dir.to_path_buf(),
            source,
        })?;
        let path = roots_dir.join(format!("R{}.json", self.round));
        let json = serde_json::to_string_pretty(self).expect("RootRecord always serializes");
        fs::write(&path, json).map_err(|source| LedgerError::Io {
            path: path.clone(),
            source,
        })?;
        Ok(path)
    }

    pub fn read(path: &Path) -> Result<RootRecord, LedgerError> {
        let text = fs::read_to_string(path).map_err(|source| LedgerError::Io {
            path: path.to_path_buf(),
            source,
        })?;
        serde_json::from_str(&text).map_err(|e| LedgerError::Malformed {
            line: 0,
            reason: e.to_string(),
        })
    }
}
