//! `contract.toml` schema (ARCHITECTURE.md §1.2).
//!
//! `properties` uses a `BTreeMap` rather than a `HashMap` so that any future
//! reserialization iterates in a stable order — the same determinism concern that drives
//! every hashing decision in `hash.rs`. Note that `code_hash` (`hash::compute_code_hash`)
//! hashes this file's *raw bytes*, not a reserialized form of this struct; this schema
//! exists for validation, not for hashing.

use serde::Deserialize;
use std::collections::BTreeMap;
use std::fs;
use std::path::{Path, PathBuf};
use thiserror::Error;

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Contract {
    #[serde(default)]
    pub io: IoSpec,
    #[serde(default)]
    pub invariants: Invariants,
    #[serde(default)]
    pub properties: BTreeMap<String, String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct IoSpec {
    pub input: Option<String>,
    pub output: Option<String>,
}

#[derive(Debug, Clone, Deserialize, Default)]
pub struct Invariants {
    #[serde(default)]
    pub pre: Vec<String>,
    #[serde(default)]
    pub post: Vec<String>,
}

#[derive(Debug, Error)]
pub enum ContractError {
    #[error("failed to read contract at {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: std::io::Error,
    },
    #[error("failed to parse contract at {path}: {source}")]
    Parse {
        path: PathBuf,
        #[source]
        source: toml::de::Error,
    },
}

pub fn load_contract(path: &Path) -> Result<Contract, ContractError> {
    let text = fs::read_to_string(path).map_err(|source| ContractError::Io {
        path: path.to_path_buf(),
        source,
    })?;
    toml::from_str(&text).map_err(|source| ContractError::Parse {
        path: path.to_path_buf(),
        source,
    })
}
