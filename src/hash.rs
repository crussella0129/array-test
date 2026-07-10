//! Content addressing primitives: `code_hash` and `cell_key` (ARCHITECTURE.md §1.1, §2).
//!
//! Two rules make every key unambiguous by construction:
//!
//! 1. **Fixed-length parts.** Combinations concatenate 32-byte digests, never raw
//!    variable-length inputs, so there is no field-boundary ambiguity (`("ab","c")` vs
//!    `("a","bc")` cannot collide).
//! 2. **Domain separation.** Every hash is derived under a named context via BLAKE3's
//!    `derive_key` mode (see [`domain`]), and leaf hashes ([`Hash::leaf`]) are
//!    constructed differently from interior nodes ([`Hash::node`]). The same bytes
//!    hashed in two domains yield unrelated digests, so a file-entry node can never be
//!    confused with a code-hash root, a leaf can never impersonate a node, and so on —
//!    the class of Merkle second-preimage confusions behind Bitcoin's CVE-2012-2459 and
//!    the leaf/node prefixes in RFC 6962 (Certificate Transparency).
//!
//! Context strings are versioned and FROZEN: once a ledger root commits to hashes
//! derived under `array-test/v1/...`, changing any context (or any structural rule in
//! this module) is a re-key event — a new version namespace and a full re-confirmation
//! of the array. See decisions.md D9.

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};
use thiserror::Error;

pub const HASH_LEN: usize = 32;

/// Frozen derivation contexts. Add new ones freely; never change or remove one that a
/// ledger has committed to.
pub mod domain {
    pub const FILE_PATH: &str = "array-test/v1/file-path";
    pub const FILE_CONTENT: &str = "array-test/v1/file-content";
    pub const FILE_ENTRY: &str = "array-test/v1/file-entry";
    pub const CONTRACT: &str = "array-test/v1/contract";
    pub const CODE_HASH: &str = "array-test/v1/code-hash";
    pub const DEPS_LIST: &str = "array-test/v1/deps-list";
    pub const SEED: &str = "array-test/v1/seed";
    pub const CELL_KEY: &str = "array-test/v1/cell-key";
    pub const TEST_DEF: &str = "array-test/v1/test-def";
    pub const FIXTURES: &str = "array-test/v1/fixtures";
    pub const EVIDENCE: &str = "array-test/v1/evidence";
    pub const LEDGER_ENTRY: &str = "array-test/v1/ledger-entry";
    pub const LEDGER_GENESIS: &str = "array-test/v1/ledger-genesis";
    pub const ROOT_LEAF: &str = "array-test/v1/root-leaf";
    pub const ARRAY_ROOT: &str = "array-test/v1/array-root";
}

/// A blake3 digest.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Hash([u8; HASH_LEN]);

impl Hash {
    /// Raw, un-domained digest. Only for opaque material whose domain lives elsewhere
    /// (e.g. caller-provided `test_def`/`fixtures`/`toolchain` hashes, test stand-ins).
    /// Production code in this crate must prefer [`Hash::leaf`]/[`Hash::node`].
    pub fn of(data: &[u8]) -> Self {
        Hash(*blake3::hash(data).as_bytes())
    }

    /// Domain-separated leaf: hashes raw bytes under `context`, prefixed with the leaf
    /// role byte `0x00` (RFC 6962 style) so a leaf can never collide with a node even
    /// if a context were ever misused in both roles.
    pub fn leaf(context: &str, data: &[u8]) -> Self {
        let mut hasher = blake3::Hasher::new_derive_key(context);
        hasher.update(&[0x00]);
        hasher.update(data);
        Hash(*hasher.finalize().as_bytes())
    }

    /// Domain-separated interior node: combines already-derived digests, in order,
    /// under `context`, prefixed with the node role byte `0x01`. Order is significant
    /// by design.
    pub fn node(context: &str, parts: &[Hash]) -> Self {
        let mut hasher = blake3::Hasher::new_derive_key(context);
        hasher.update(&[0x01]);
        for part in parts {
            hasher.update(&part.0);
        }
        Hash(*hasher.finalize().as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; HASH_LEN] {
        &self.0
    }

    /// Bare hex, no `blake3:` prefix — safe for filenames on every platform.
    pub fn hex(&self) -> String {
        let mut s = String::with_capacity(HASH_LEN * 2);
        for b in &self.0 {
            s.push_str(&format!("{:02x}", b));
        }
        s
    }
}

impl fmt::Display for Hash {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "blake3:")?;
        for b in &self.0 {
            write!(f, "{:02x}", b)?;
        }
        Ok(())
    }
}

#[derive(Debug, Error, PartialEq, Eq)]
#[error("invalid hash string: {0}")]
pub struct ParseHashError(String);

impl std::str::FromStr for Hash {
    type Err = ParseHashError;

    fn from_str(s: &str) -> Result<Self, Self::Err> {
        let hex = s
            .strip_prefix("blake3:")
            .ok_or_else(|| ParseHashError(format!("missing 'blake3:' prefix in {s:?}")))?;
        if hex.len() != HASH_LEN * 2 {
            return Err(ParseHashError(format!(
                "expected {} hex chars, got {}",
                HASH_LEN * 2,
                hex.len()
            )));
        }
        let mut bytes = [0u8; HASH_LEN];
        for (i, byte) in bytes.iter_mut().enumerate() {
            *byte = u8::from_str_radix(&hex[i * 2..i * 2 + 2], 16)
                .map_err(|_| ParseHashError(format!("non-hex characters in {s:?}")))?;
        }
        Ok(Hash(bytes))
    }
}

impl serde::Serialize for Hash {
    fn serialize<S: serde::Serializer>(&self, serializer: S) -> Result<S::Ok, S::Error> {
        serializer.serialize_str(&self.to_string())
    }
}

impl<'de> serde::Deserialize<'de> for Hash {
    fn deserialize<D: serde::Deserializer<'de>>(deserializer: D) -> Result<Self, D::Error> {
        let s = <String as serde::Deserialize>::deserialize(deserializer)?;
        s.parse().map_err(serde::de::Error::custom)
    }
}

#[derive(Debug, Error)]
pub enum CodeHashError {
    #[error("io error under {path}: {source}")]
    Io {
        path: PathBuf,
        #[source]
        source: io::Error,
    },
    #[error("non-UTF-8 file name in unit source tree: {path:?} (names must be UTF-8 so hashes are collision-free and portable)")]
    NonUtf8Path { path: PathBuf },
    #[error("symlink in unit source tree: {path} (symlinks can point outside the unit, breaking hermeticity)")]
    Symlink { path: PathBuf },
}

/// A file inside a unit's `src/`, addressed by its normalized relative path.
struct SrcFile {
    /// `/`-joined UTF-8 relative path — identical on every platform.
    normalized: String,
    absolute: PathBuf,
}

/// Recursively collect every file under `dir`, sorted by normalized relative path.
/// Sorting by the normalized *string* (not `PathBuf`) makes ordering — and therefore
/// the resulting hash — independent of both filesystem iteration order and platform
/// path encoding. Symlinks and non-UTF-8 names are rejected, not followed or lossily
/// converted: both would let two different trees hash identically (or one tree hash
/// differently across platforms), which the provability claim cannot tolerate.
fn collect_src_files(dir: &Path) -> Result<Vec<SrcFile>, CodeHashError> {
    fn walk(
        current: &Path,
        rel_parts: &mut Vec<String>,
        out: &mut Vec<SrcFile>,
    ) -> Result<(), CodeHashError> {
        let entries = fs::read_dir(current).map_err(|source| CodeHashError::Io {
            path: current.to_path_buf(),
            source,
        })?;
        for entry in entries {
            let entry = entry.map_err(|source| CodeHashError::Io {
                path: current.to_path_buf(),
                source,
            })?;
            let path = entry.path();
            let file_type = entry.file_type().map_err(|source| CodeHashError::Io {
                path: path.clone(),
                source,
            })?;
            if file_type.is_symlink() {
                return Err(CodeHashError::Symlink { path });
            }
            let name = entry
                .file_name()
                .into_string()
                .map_err(|_| CodeHashError::NonUtf8Path { path: path.clone() })?;
            rel_parts.push(name);
            if file_type.is_dir() {
                walk(&path, rel_parts, out)?;
            } else {
                out.push(SrcFile {
                    normalized: rel_parts.join("/"),
                    absolute: path,
                });
            }
            rel_parts.pop();
        }
        Ok(())
    }

    let mut out = Vec::new();
    walk(dir, &mut Vec::new(), &mut out)?;
    out.sort_by(|a, b| a.normalized.cmp(&b.normalized));
    Ok(out)
}

/// `code_hash = H(src/ ‖ contract.toml)` (ARCHITECTURE.md §1.1).
///
/// `unit_dir` must contain a `src/` directory and a `contract.toml` file. Each file
/// under `src/` contributes a `FILE_ENTRY` node over its (`FILE_PATH` leaf,
/// `FILE_CONTENT` leaf) pair — so renames and content edits are both detected — and the
/// root is a `CODE_HASH` node over the `CONTRACT` leaf followed by the entries in
/// normalized-path order. See [`collect_src_files`] for what is rejected and why.
pub fn compute_code_hash(unit_dir: &Path) -> Result<Hash, CodeHashError> {
    let src_dir = unit_dir.join("src");
    let contract_path = unit_dir.join("contract.toml");

    let contract_bytes = fs::read(&contract_path).map_err(|source| CodeHashError::Io {
        path: contract_path.clone(),
        source,
    })?;
    let contract_hash = Hash::leaf(domain::CONTRACT, &contract_bytes);

    let mut parts = vec![contract_hash];
    for file in collect_src_files(&src_dir)? {
        let contents = fs::read(&file.absolute).map_err(|source| CodeHashError::Io {
            path: file.absolute.clone(),
            source,
        })?;
        let path_hash = Hash::leaf(domain::FILE_PATH, file.normalized.as_bytes());
        let content_hash = Hash::leaf(domain::FILE_CONTENT, &contents);
        parts.push(Hash::node(domain::FILE_ENTRY, &[path_hash, content_hash]));
    }

    Ok(Hash::node(domain::CODE_HASH, &parts))
}

/// Inputs to a cell's `cell_key` (ARCHITECTURE.md §2). `scope_dep_hashes_in_dag_order`
/// must be supplied in a deterministic order — use [`crate::dag::Dag::topo_order`] — and
/// the same order must be used every time this cell is keyed, or the key will spuriously
/// change.
pub struct CellKeyInputs<'a> {
    pub target_code_hash: Hash,
    pub scope_dep_hashes_in_dag_order: &'a [Hash],
    pub test_def_hash: Hash,
    pub fixtures_hash: Hash,
    pub seed: u64,
    pub toolchain_hash: Hash,
}

/// `cell_key = H(target.code_hash ‖ H(scope deps) ‖ test_def_hash ‖ fixtures_hash ‖ seed
/// ‖ toolchain_hash)` (ARCHITECTURE.md §2).
pub fn compute_cell_key(inputs: &CellKeyInputs) -> Hash {
    let deps_hash = Hash::node(domain::DEPS_LIST, inputs.scope_dep_hashes_in_dag_order);
    let seed_hash = Hash::leaf(domain::SEED, &inputs.seed.to_le_bytes());
    Hash::node(
        domain::CELL_KEY,
        &[
            inputs.target_code_hash,
            deps_hash,
            inputs.test_def_hash,
            inputs.fixtures_hash,
            seed_hash,
            inputs.toolchain_hash,
        ],
    )
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn given_the_same_bytes_in_two_domains_should_produce_different_hashes() {
        let a = Hash::leaf(domain::FILE_PATH, b"identical");
        let b = Hash::leaf(domain::FILE_CONTENT, b"identical");
        assert_ne!(a, b);
    }

    #[test]
    fn given_a_node_and_a_leaf_over_identical_bytes_should_differ() {
        // A node over [h1, h2] feeds h1.bytes ++ h2.bytes to the hasher; a leaf over
        // those same 64 bytes in the same context must still differ — the 0x00/0x01
        // role prefixes guarantee it structurally, independent of the context table.
        let h1 = Hash::of(b"one");
        let h2 = Hash::of(b"two");
        let node = Hash::node(domain::CODE_HASH, &[h1, h2]);

        let mut concatenated = Vec::new();
        concatenated.extend_from_slice(h1.as_bytes());
        concatenated.extend_from_slice(h2.as_bytes());
        let leaf = Hash::leaf(domain::CODE_HASH, &concatenated);

        assert_ne!(node, leaf);
    }

    #[test]
    fn given_empty_inputs_in_different_domains_should_differ() {
        let a = Hash::node(domain::DEPS_LIST, &[]);
        let b = Hash::node(domain::CELL_KEY, &[]);
        assert_ne!(a, b);
    }
}
