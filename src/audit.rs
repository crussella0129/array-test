//! The full state audit (D19): everything D11 promises is independently verifiable,
//! verified. Library-first — the CLI's `verify` is just one caller of [`full_audit`];
//! an embedder holding a state dir can run the same audit with zero trust in the
//! runner that produced it.

use crate::hash::{domain, Hash};
use crate::judge::read_judgments;
use crate::ledger::{load_and_verify, LedgerEntry, RootRecord};
use crate::round::StatePaths;
use std::collections::BTreeMap;
use std::fs;
use std::path::Path;

/// The audit's outcome. `problems` are integrity violations (a nonzero-exit matter);
/// `notes` are informational. Success must never read as failure and vice versa — the
/// two lists are never mixed (s8 research §1).
#[derive(Debug, Default)]
pub struct AuditReport {
    pub problems: Vec<String>,
    pub notes: Vec<String>,
    pub confirmations: usize,
    pub roots_checked: usize,
    pub judgments: usize,
    pub evidence_files: usize,
}

impl AuditReport {
    pub fn clean(&self) -> bool {
        self.problems.is_empty()
    }
}

/// Audit every verifiable surface of a state dir: the confirmations chain, every root
/// certificate, the judgments chain, and the evidence store.
pub fn full_audit(state_dir: &Path) -> AuditReport {
    let paths = StatePaths::new(state_dir);
    let mut report = AuditReport::default();

    // 1. Confirmations chain (every entry re-hashed, every link).
    let entries: Vec<LedgerEntry> = if paths.ledger_file.exists() {
        match load_and_verify(&paths.ledger_file) {
            Ok(entries) => {
                report.confirmations = entries.len();
                entries
            }
            Err(e) => {
                report.problems.push(format!("confirmations ledger: {e}"));
                Vec::new()
            }
        }
    } else {
        report.notes.push("no confirmations ledger present".into());
        Vec::new()
    };

    // 2. Every root certificate must match a recomputation from its round's entries.
    let mut by_round: BTreeMap<u32, Vec<LedgerEntry>> = BTreeMap::new();
    for entry in &entries {
        by_round.entry(entry.round).or_default().push(entry.clone());
    }
    if paths.roots_dir.exists() {
        let mut root_files: Vec<_> = match fs::read_dir(&paths.roots_dir) {
            Ok(dir) => dir.filter_map(|e| e.ok().map(|e| e.path())).collect(),
            Err(e) => {
                report.problems.push(format!("roots dir unreadable: {e}"));
                Vec::new()
            }
        };
        root_files.sort();
        for path in root_files {
            let Ok(record) = RootRecord::read(&path) else {
                report
                    .problems
                    .push(format!("unreadable root certificate {}", path.display()));
                continue;
            };
            let Some(round_entries) = by_round.get(&record.round) else {
                report.problems.push(format!(
                    "root certificate R{} has no ledger entries",
                    record.round
                ));
                continue;
            };
            let recomputed = RootRecord::from_entries(record.round, round_entries);
            if recomputed != record {
                report.problems.push(format!(
                    "root certificate R{} does not match the ledger (certificate root {}, ledger root {})",
                    record.round, record.root, recomputed.root
                ));
            }
            report.roots_checked += 1;
        }
    }

    // F16: rounds present in the ledger but lacking a certificate (e.g. a crash
    // between ledger-append and certificate-write) are worth a note — not an
    // integrity violation, but silence isn't a note either.
    let mut uncertified: Vec<u32> = by_round
        .keys()
        .filter(|round| !paths.roots_dir.join(format!("R{round}.json")).exists())
        .copied()
        .collect();
    uncertified.sort_unstable();
    if !uncertified.is_empty() {
        report.notes.push(format!(
            "ledger round(s) without a certificate: {}",
            uncertified
                .iter()
                .map(|r| format!("R{r}"))
                .collect::<Vec<_>>()
                .join(", ")
        ));
    }

    // 3. Judgments chain (§7.3): audited even though never rooted.
    match read_judgments(&paths) {
        Ok(judgments) => report.judgments = judgments.len(),
        Err(e) => report.problems.push(format!("judgments ledger: {e}")),
    }

    // 4. Evidence store: content-addressed, so every file must re-hash to its name.
    if paths.evidence_dir.exists() {
        match fs::read_dir(&paths.evidence_dir) {
            Ok(dir) => {
                for entry in dir.filter_map(|e| e.ok()) {
                    let path = entry.path();
                    let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                        continue;
                    };
                    let Ok(bytes) = fs::read(&path) else {
                        report
                            .problems
                            .push(format!("unreadable evidence file {}", path.display()));
                        continue;
                    };
                    let actual = Hash::leaf(domain::EVIDENCE, &bytes);
                    if actual.hex() != stem {
                        report.problems.push(format!(
                            "evidence file {} does not match its content address",
                            path.display()
                        ));
                    }
                    report.evidence_files += 1;
                }
            }
            Err(e) => report
                .problems
                .push(format!("evidence dir unreadable: {e}")),
        }
    }
    let stored: std::collections::BTreeSet<String> = fs::read_dir(&paths.evidence_dir)
        .map(|dir| {
            dir.filter_map(|e| e.ok())
                .filter_map(|e| {
                    e.path()
                        .file_stem()
                        .and_then(|s| s.to_str())
                        .map(String::from)
                })
                .collect()
        })
        .unwrap_or_default();
    let missing = entries
        .iter()
        .filter(|e| !stored.contains(&e.evidence_hash.hex()))
        .count();
    if missing > 0 {
        // Informational: quarantined/skipped evidence is legitimately never stored.
        report.notes.push(format!(
            "{missing} ledger entr{} without stored evidence (quarantined/skipped/legacy)",
            if missing == 1 { "y" } else { "ies" }
        ));
    }

    report
}
