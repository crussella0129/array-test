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
