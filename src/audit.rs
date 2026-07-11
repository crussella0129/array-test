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
    pub mutations: usize,
    pub fuzz_entries: usize,
    pub evidence_files: usize,
}

impl AuditReport {
    pub fn clean(&self) -> bool {
        self.problems.is_empty()
    }
}

/// Audit every verifiable surface of a state dir: the confirmations chain, every root
/// certificate, the sidecar chains (judgments/mutations/fuzz), and the evidence store.
///
/// Each surface is an independent phase over one shared [`AuditReport`]; the phases are
/// factored out below so each can be read — and tested — on its own (F3). Order matters
/// only in that later phases reuse the verified `entries` the first phase returns.
pub fn full_audit(state_dir: &Path) -> AuditReport {
    let paths = StatePaths::new(state_dir);
    let mut report = AuditReport::default();

    let entries = audit_confirmations(&paths, &mut report);
    audit_roots(&paths, &mut report, &entries);
    audit_sidecar_chains(&paths, &mut report);
    audit_evidence(&paths, &mut report, &entries);

    report
}

/// Phase 1: the confirmations chain — every entry re-hashed, every link checked. Returns
/// the verified entries (empty on absence or failure) for the later phases to reuse.
fn audit_confirmations(paths: &StatePaths, report: &mut AuditReport) -> Vec<LedgerEntry> {
    if !paths.ledger_file.exists() {
        report.notes.push("no confirmations ledger present".into());
        return Vec::new();
    }
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
}

/// Phase 2: every root certificate must match a recomputation from its round's entries,
/// and every ledger round should have a certificate (a missing one is a note, not a
/// violation — a crash between ledger-append and certificate-write leaves exactly that).
fn audit_roots(paths: &StatePaths, report: &mut AuditReport, entries: &[LedgerEntry]) {
    let mut by_round: BTreeMap<u32, Vec<LedgerEntry>> = BTreeMap::new();
    for entry in entries {
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

    // F16: rounds present in the ledger but lacking a certificate are worth a note —
    // not an integrity violation, but silence isn't a note either.
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
}

/// Phase 3: the sidecar chains — judgments (§7.3), mutations (T12/D23), fuzz (T13/D24).
/// Each is audited even though none is rooted; a broken chain is a problem, a count note.
fn audit_sidecar_chains(paths: &StatePaths, report: &mut AuditReport) {
    match read_judgments(paths) {
        Ok(judgments) => report.judgments = judgments.len(),
        Err(e) => report.problems.push(format!("judgments ledger: {e}")),
    }
    match crate::mutation::read_mutations(paths) {
        Ok(mutations) => report.mutations = mutations.len(),
        Err(e) => report.problems.push(format!("mutations ledger: {e}")),
    }
    match crate::fuzz::read_fuzz_entries(paths) {
        Ok(entries) => report.fuzz_entries = entries.len(),
        Err(e) => report.problems.push(format!("fuzz ledger: {e}")),
    }
}

/// Phase 4: the evidence store — content-addressed, so every file must re-hash to its
/// name; conversely a ledger entry without stored evidence is a note (quarantined/skipped
/// evidence is legitimately never stored), not a violation.
fn audit_evidence(paths: &StatePaths, report: &mut AuditReport, entries: &[LedgerEntry]) {
    // Single enumeration of the store (was two): hash-check each file AND collect its
    // content-address stem in the same pass, so the "missing evidence" note below can be
    // answered without re-reading the directory.
    let mut stored: std::collections::BTreeSet<String> = std::collections::BTreeSet::new();
    if paths.evidence_dir.exists() {
        match fs::read_dir(&paths.evidence_dir) {
            Ok(dir) => {
                for entry in dir.filter_map(std::result::Result::ok) {
                    let path = entry.path();
                    let Some(stem) = path.file_stem().and_then(|s| s.to_str()) else {
                        continue;
                    };
                    stored.insert(stem.to_string());
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
    let missing = entries
        .iter()
        .filter(|e| !stored.contains(&e.evidence_hash.hex()))
        .count();
    if missing > 0 {
        report.notes.push(format!(
            "{missing} ledger entr{} without stored evidence (quarantined/skipped/legacy)",
            if missing == 1 { "y" } else { "ies" }
        ));
    }
}
