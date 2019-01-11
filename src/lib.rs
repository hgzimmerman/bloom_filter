pub mod bloom_filter;
pub mod w_lock_bloom_filter;
pub mod hash_numbers;
pub mod counting_bloom_filter;
pub mod rehasher;
pub mod counting_w_lock_bloom_filter;
pub mod hash_to_indicies;

pub use crate::bloom_filter::BloomFilter;

/// Calculates the ideal false positive rate.
/// If the hashing functions that are used in a bloom filter produce a non-uniform distribution of hashes
/// then the actual false positive rate should be higher than stated.
///
/// k: number of hash functions
/// n: number of elements
/// m: number of bits
pub fn false_positive_rate(k: usize, n: usize, m: usize ) -> f64 {
    use std::f64::consts::E;
    (1.0 - E.powf(((0-k as isize)*n as isize) as f64/m as f64)).powi(k as i32)
}

/// Note: This is irrespective of K,
/// This also assumes an optimal K value
/// m = ceil((n * log(p)) / log(1 / pow(2, log(2))))
pub fn needed_size(n: usize, p: f64) -> usize {
    ((n as f64 * p.ln()) / (1.0 / 2f64.powf(2f64.ln())).ln()).ceil() as usize
}

pub fn optimal_k(n: usize, m: usize) -> usize {
    ((m/n) as f64 * 2f64.ln()).ceil() as usize
}


#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn needed_size1() {
        let m = needed_size(2000, 0.001);
        assert_eq!(m, 28756)
    }

    #[test]
    fn optimal_k1() {
        let m = 28756;
        let k = optimal_k(2000, m);
        assert_eq!(k, 10)
    }

}