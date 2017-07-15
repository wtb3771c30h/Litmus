//! Deterministic hashing and pseudo-random utilities (stdlib only).
//!
//! We avoid `std::collections::hash_map::DefaultHasher` for fingerprints
//! because its output is not guaranteed stable across Rust versions. FNV-1a
//! is tiny, dependency-free, and stable, which is exactly what we want when
//! persisting fingerprints into JSON evidence files.

/// 64-bit FNV-1a hash of a byte slice, rendered as lowercase hex.
pub fn fingerprint(bytes: &[u8]) -> String {
    const OFFSET: u64 = 0xcbf29ce484222325;
    const PRIME: u64 = 0x00000100000001b3;
    let mut hash = OFFSET;
    for &b in bytes {
        hash ^= b as u64;
        hash = hash.wrapping_mul(PRIME);
    }
    format!("{hash:016x}")
}

/// A minimal SplitMix64 PRNG. Deterministic given a seed; used for env-order
/// shuffling and any controlled jitter so that runs are reproducible.
#[derive(Debug, Clone)]
pub struct Rng {
    state: u64,
}

impl Rng {
