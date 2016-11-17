//! Deterministic hashing and pseudo-random utilities (stdlib only).
//!
//! We avoid `std::collections::hash_map::DefaultHasher` for fingerprints
//! because its output is not guaranteed stable across Rust versions. FNV-1a
//! is tiny, dependency-free, and stable, which is exactly what we want when
