//! Deterministic hashing and pseudo-random utilities (stdlib only).
//!
//! We avoid `std::collections::hash_map::DefaultHasher` for fingerprints
//! because its output is not guaranteed stable across Rust versions. FNV-1a
//! is tiny, dependency-free, and stable, which is exactly what we want when
//! persisting fingerprints into JSON evidence files.

/// 64-bit FNV-1a hash of a byte slice, rendered as lowercase hex.
pub fn fingerprint(bytes: &[u8]) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
