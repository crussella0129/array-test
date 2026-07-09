//! Content addressing primitives: `code_hash` and `cell_key` (ARCHITECTURE.md §1.1, §2).
//!
//! Every combination below concatenates fixed-length (32-byte) digests rather than raw,
//! variable-length inputs, so there is no path/content or field-boundary ambiguity to
//! exploit (e.g. `("ab", "c")` and `("a", "bc")` cannot collide).

use std::fmt;
use std::fs;
use std::io;
use std::path::{Path, PathBuf};

pub const HASH_LEN: usize = 32;

/// A blake3 digest.
#[derive(Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Debug)]
pub struct Hash([u8; HASH_LEN]);

impl Hash {
    pub fn of(data: &[u8]) -> Self {
        Hash(*blake3::hash(data).as_bytes())
    }

    /// Domain-separated combination of already-hashed digests, in the given order.
    pub fn combine(parts: &[Hash]) -> Self {
        let mut hasher = blake3::Hasher::new();
        for part in parts {
            hasher.update(&part.0);
        }
        Hash(*hasher.finalize().as_bytes())
    }

    pub fn as_bytes(&self) -> &[u8; HASH_LEN] {
        &self.0
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

/// Recursively list every file under `dir`, as paths relative to `dir`, in a stable
/// sorted order. Sorting is what makes the resulting hash independent of filesystem
/// iteration order.
pub fn list_files_sorted(dir: &Path) -> io::Result<Vec<PathBuf>> {
    fn walk(base: &Path, current: &Path, out: &mut Vec<PathBuf>) -> io::Result<()> {
        for entry in fs::read_dir(current)? {
            let entry = entry?;
            let path = entry.path();
            if path.is_dir() {
                walk(base, &path, out)?;
            } else {
                out.push(path.strip_prefix(base).unwrap().to_path_buf());
            }
        }
        Ok(())
    }

    let mut out = Vec::new();
    walk(dir, dir, &mut out)?;
    out.sort();
    Ok(out)
}

/// `code_hash = H(src/ ‖ contract.toml)` (ARCHITECTURE.md §1.1).
///
/// `unit_dir` must contain a `src/` directory and a `contract.toml` file. Each file
/// under `src/` contributes a leaf hash over its (relative path, contents) pair, so
/// renames and content edits are both detected; leaves are combined in sorted-path
/// order together with the contract's hash.
pub fn compute_code_hash(unit_dir: &Path) -> io::Result<Hash> {
    let src_dir = unit_dir.join("src");
    let contract_path = unit_dir.join("contract.toml");

    let contract_bytes = fs::read(&contract_path)?;
    let contract_hash = Hash::of(&contract_bytes);

    let mut leaves = vec![contract_hash];
    for rel_path in list_files_sorted(&src_dir)? {
        let contents = fs::read(src_dir.join(&rel_path))?;
        let path_hash = Hash::of(rel_path.to_string_lossy().as_bytes());
        let content_hash = Hash::of(&contents);
        leaves.push(Hash::combine(&[path_hash, content_hash]));
    }

    Ok(Hash::combine(&leaves))
}

/// Inputs to a cell's `cell_key` (ARCHITECTURE.md §2). `scope_dep_hashes_in_dag_order`
/// must be supplied in a caller-determined deterministic order (e.g. sorted by unit id,
/// or DAG topological order) — the same order must be used every time this cell is
/// keyed, or the key will spuriously change.
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
    let deps_hash = Hash::combine(inputs.scope_dep_hashes_in_dag_order);
    let seed_hash = Hash::of(&inputs.seed.to_le_bytes());
    Hash::combine(&[
        inputs.target_code_hash,
        deps_hash,
        inputs.test_def_hash,
        inputs.fixtures_hash,
        seed_hash,
        inputs.toolchain_hash,
    ])
}
