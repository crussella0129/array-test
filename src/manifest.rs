//! `manifest.toml` schema (ARCHITECTURE.md §1.1).
//!
//! `code_hash` is deliberately absent from this struct: it is computed
//! (`hash::compute_code_hash`), never authored, so it can never drift from the content it
//! is supposed to address.

use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Deserialize)]
pub struct Manifest {
    pub id: String,
    pub sprint: u32,
    pub version: String,
    #[serde(default)]
    pub deps: Vec<String>,
    /// Legacy single test declaration — sugar for `[tests.closure]` (D15). Declaring
    /// both is a validation error.
    #[serde(default)]
    pub test: Option<TestSpec>,
    /// Per-scope tests: keys are `unit` | `direct` | `closure` | `e2e` (D15). A unit
    /// with `[tests.e2e]` is thereby an entrypoint declaration (§1.4). Units without
    /// any test contribute code (and dep hashes) but no cell of their own.
    #[serde(default)]
    pub tests: std::collections::BTreeMap<String, TestSpec>,
}

pub const VALID_SCOPES: &[&str] = &["unit", "direct", "closure", "e2e"];

#[derive(Debug, Clone, Deserialize)]
pub struct TestSpec {
    /// argv vector; `command[0]` is the program. No shell is implied.
    pub command: Vec<String>,
    /// Declared environment for the cell (D12: only declared vars reach the child).
    #[serde(default)]
    pub env: std::collections::BTreeMap<String, String>,
    /// Wall-clock envelope override in seconds (per-scope defaults apply otherwise).
    pub timeout_secs: Option<u64>,
    /// Opt-in memory cap (RLIMIT_AS) in megabytes (T3b).
    pub mem_limit_mb: Option<u64>,
}

#[derive(Debug, Error)]
pub enum ManifestError {
    #[error("failed to read manifest at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse manifest at {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
    #[error("invalid manifest at {path}: {reason}")]
    Invalid { path: PathBuf, reason: String },
}

impl Manifest {
    /// Structural validation beyond what the type system enforces. Rejecting these at
    /// load time keeps errors close to their cause — a self-dependency surfacing later
    /// as a DAG cycle error names the symptom, not the mistake.
    fn validate(&self) -> Result<(), String> {
        if self.id.trim().is_empty() {
            return Err("id must be non-empty".to_string());
        }
        let mut seen = std::collections::BTreeSet::new();
        for dep in &self.deps {
            if dep == &self.id {
                return Err(format!("unit '{}' depends on itself", self.id));
            }
            if !seen.insert(dep) {
                return Err(format!("duplicate dependency '{dep}'"));
            }
        }
        if let Some(test) = &self.test {
            if test.command.is_empty() {
                return Err("test.command must be non-empty when [test] is declared".to_string());
            }
        }
        for (scope, spec) in &self.tests {
            if !VALID_SCOPES.contains(&scope.as_str()) {
                return Err(format!(
                    "unknown test scope '{scope}' (expected one of: unit, direct, closure, e2e)"
                ));
            }
            if spec.command.is_empty() {
                return Err(format!("tests.{scope}.command must be non-empty"));
            }
        }
        if self.test.is_some() && self.tests.contains_key("closure") {
            return Err(
                "declare either legacy [test] or [tests.closure], not both (they are the same scope)"
                    .to_string(),
            );
        }
        Ok(())
    }
}

pub fn load_manifest(path: &Path) -> Result<Manifest, ManifestError> {
    let text = fs::read_to_string(path).map_err(|source| ManifestError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    let manifest: Manifest = toml::from_str(&text).map_err(|source| ManifestError::Parse {
        path: path.to_path_buf(),
        source,
    })?;
    manifest.validate().map_err(|reason| ManifestError::Invalid {
        path: path.to_path_buf(),
        reason,
    })?;
    Ok(manifest)
}
