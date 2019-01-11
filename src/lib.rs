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
