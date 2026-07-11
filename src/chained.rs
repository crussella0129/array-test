//! Shared mechanics for the hash-chained append-only sidecars (F1). The four ledgers
//! (confirmations, judgments, mutations, fuzz) each own their entry type and canonical
//! byte layout, but the *chaining* — derive `(seq, prev)` from the tail, append a line,
//! advance in memory — is identical. Centralizing it here fixes the O(N²)
//! re-read-on-every-append bug that `mutation`/`fuzz` had (they called `read_*` per
//! append); a writer built on [`ChainState`] reads the tail once at open and keeps
//! `(last_hash, next_seq)` in memory thereafter.
//!
//! This is deliberately *not* a generic-over-entry-type ledger: the four entry layouts
//! differ (and one, confirmations, is freeze-locked by the durable ledger), so a shared
//! trait would carry more machinery than a 4-instance pattern earns. The bookkeeping
//! primitive is the part worth sharing.

use crate::hash::Hash;
use std::fs::{self, OpenOptions};
use std::io::Write;
use std::path::Path;

/// In-memory chain position: the previous entry's hash and the next sequence number.
#[derive(Debug, Clone, Copy)]
pub(crate) struct ChainState {
    last_hash: Hash,
    next_seq: u64,
}

impl ChainState {
    /// Start a chain whose tail is `last` (entry_hash, seq) or, if empty, `genesis`.
    pub(crate) fn new(last: Option<(Hash, u64)>, genesis: Hash) -> Self {
        let (last_hash, next_seq) = last.map_or((genesis, 0), |(h, seq)| (h, seq + 1));
        ChainState {
            last_hash,
            next_seq,
        }
    }

    /// The `(seq, prev)` the next appended entry must use.
    pub(crate) fn next(&self) -> (u64, Hash) {
        (self.next_seq, self.last_hash)
    }

    /// Record that an entry with `entry_hash` was appended.
    pub(crate) fn advance(&mut self, entry_hash: Hash) {
        self.last_hash = entry_hash;
        self.next_seq += 1;
    }
}

/// Append one ndjson line to `path`, creating the parent dir and file if needed.
pub(crate) fn append_ndjson_line(path: &Path, line: &str) -> std::io::Result<()> {
    if let Some(parent) = path.parent() {
        fs::create_dir_all(parent)?;
    }
    let mut file = OpenOptions::new().create(true).append(true).open(path)?;
    writeln!(file, "{line}")
}
