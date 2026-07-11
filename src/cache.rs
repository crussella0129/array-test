//! Content-addressed cache reads, made observable (F6). Four sites (round, judge,
//! mutation, fuzz) previously read a JSON cache file and treated *any* failure as
//! "not cached" — one via `read_to_string(...).unwrap_or_default()`, which turns a
//! permission-denied into `""` and hides real I/O errors. This one helper makes the
//! distinction: `NotFound` is a legitimate miss (silent); anything else is surfaced on
//! stderr before falling back to a miss, so cache corruption cannot stay invisible.

use serde::de::DeserializeOwned;
use std::path::Path;

/// Read a JSON cache entry. `None` = a genuine miss (absent, or unreadable/corrupt with
/// a diagnostic already printed). Never conflates "absent" with "broken".
pub(crate) fn read_cache<T: DeserializeOwned>(path: &Path) -> Option<T> {
    let text = match std::fs::read_to_string(path) {
        Ok(t) => t,
        Err(e) if e.kind() == std::io::ErrorKind::NotFound => return None,
        Err(e) => {
            eprintln!("[cache] unreadable {}: {e}", path.display());
            return None;
        }
    };
    match serde_json::from_str(&text) {
        Ok(v) => Some(v),
        Err(e) => {
            eprintln!("[cache] corrupt {}: {e}", path.display());
            None
        }
    }
}
