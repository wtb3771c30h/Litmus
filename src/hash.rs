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
    pub fn new(seed: u64) -> Rng {
        // Avoid the degenerate all-zero state.
        Rng {
            state: seed ^ 0x9e3779b97f4a7c15,
        }
    }

    /// Next raw 64-bit value.
    pub fn next_u64(&mut self) -> u64 {
        self.state = self.state.wrapping_add(0x9e3779b97f4a7c15);
        let mut z = self.state;
        z = (z ^ (z >> 30)).wrapping_mul(0xbf58476d1ce4e5b9);
        z = (z ^ (z >> 27)).wrapping_mul(0x94d049bb133111eb);
        z ^ (z >> 31)
    }

    /// Uniform integer in `[0, bound)`. Returns 0 when `bound == 0`.
    pub fn below(&mut self, bound: usize) -> usize {
        if bound == 0 {
            return 0;
        }
        (self.next_u64() % bound as u64) as usize
    }

    /// In-place Fisher-Yates shuffle.
    pub fn shuffle<T>(&mut self, items: &mut [T]) {
        let len = items.len();
        if len <= 1 {
            return;
        }
        for i in (1..len).rev() {
            let j = self.below(i + 1);
            items.swap(i, j);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn fnv_is_stable() {
        // Known FNV-1a/64 value for the empty input is the offset basis.
        assert_eq!(fingerprint(b""), "cbf29ce484222325");
        // Same input always hashes the same.
        assert_eq!(fingerprint(b"litmus"), fingerprint(b"litmus"));
        assert_ne!(fingerprint(b"a"), fingerprint(b"b"));
    }

    #[test]
    fn rng_is_deterministic() {
        let mut a = Rng::new(1234);
        let mut b = Rng::new(1234);
        for _ in 0..100 {
            assert_eq!(a.next_u64(), b.next_u64());
        }
    }

    #[test]
    fn rng_below_bounds() {
        let mut r = Rng::new(9);
        for _ in 0..1000 {
            assert!(r.below(7) < 7);
        }
        assert_eq!(r.below(0), 0);
        assert_eq!(r.below(1), 0);
    }

    #[test]
    fn shuffle_is_a_permutation() {
        let mut r = Rng::new(2024);
        let mut v: Vec<u32> = (0..50).collect();
        let original: Vec<u32> = v.clone();
        r.shuffle(&mut v);
        let mut sorted = v.clone();
        sorted.sort_unstable();
        assert_eq!(sorted, original);
        // Extremely unlikely to be identical after shuffling 50 items.
        assert_ne!(v, original);
    }
}
