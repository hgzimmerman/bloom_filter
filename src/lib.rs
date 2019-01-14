#![feature(test)]

#![cfg_attr(not(feature = "std"), no_std)]


//! Bloom filters offer time and space efficient lookup with no false negatives,
//! and with a false positive rate dependent on the number of hashers (`k`), number of entries (`n`),
//! and number of bits in the filter (`m`).
//! The false positive rate will increase as `n` rises, and will fall as `k` and `m` rise.

extern crate test;

pub mod bloom_filter;
pub mod counting_bloom_filter;
pub mod counting_w_lock_bloom_filter;
pub mod hash_numbers;
pub mod hash_to_indicies;
pub mod rehasher;
pub mod w_lock_bloom_filter;

pub use crate::bloom_filter::BloomFilter;
pub use crate::counting_bloom_filter::CountingBloomFilter;
pub use crate::counting_w_lock_bloom_filter::CountingWLockBloomFilter;
pub use crate::w_lock_bloom_filter::WLockBloomFilter;

pub use crate::rehasher::ReHasher;

/// Calculates the ideal false positive rate.
/// If the hashing functions that are used in a bloom filter produce a non-uniform distribution of hashes
/// then the actual false positive rate should be higher than stated.
///
/// k: number of hash functions
/// n: number of elements
/// m: number of bits
pub fn false_positive_rate(k: usize, n: usize, m: usize) -> f64 {
    use core::f64::consts::E;
    (1.0 - E.powf(((0 - k as isize) * n as isize) as f64 / m as f64)).powi(k as i32)
}

/// Gets the required number of bits (`m`) if given `k`, `n` and `p`.
///
/// # Note
/// This is useful if you want to choose `k` beforehand for performance reasons,
/// and you want to know how big the bloom filter will need to be to achieve a desired false positive rate.
pub fn m_from_knp(k: usize, n: usize, p: f64) -> usize {
    -((k * n) as f64 / (1f64 - p.powf(1.0 / (k as f64))).ln()) as usize
}

/// Gets the required number of bits (`m`) assuming an optimal `k`, using `n` and `p`.
pub fn optimal_m(n: usize, p: f64) -> usize {
    // m = ceil((n * log(p)) / log(1 / pow(2, log(2))))
    ((n as f64 * p.ln()) / (1.0 / 2f64.powf(2f64.ln())).ln()).ceil() as usize
}

/// This gets the optimal number of hashes to perform (`k`) given `n` and `m`.
pub fn optimal_k(n: usize, m: usize) -> usize {
    ((m / n) as f64 * 2f64.ln()).ceil() as usize
}

#[cfg(test)]
mod tests {
    use super::*;
    #[test]
    fn needed_size1() {
        let m = optimal_m(2000, 0.001);
        assert_eq!(m, 28756)
    }

    #[test]
    fn optimal_k1() {
        let m = 28756;
        let k = optimal_k(2000, m);
        assert_eq!(k, 10)
    }

    #[test]
    fn solve_for_m() {
        let p = false_positive_rate(4, 1000, 10000);
        let m = m_from_knp(4, 1000, p);
        assert_eq!(m, 10000)
    }
}

#[cfg(test)]
mod benches {
    use super::*;
    use test::Bencher;
    use murmur3::murmur3_32::MurmurHasher;
    mod thousand_elements {
        use super::*;
        mod normal {
            use super::*;

            #[bench]
            fn optimal_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = BloomFilter::<i32, ReHasher<MurmurHasher>>::optimal_new(1000, 0.001);
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k7_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = BloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k4_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = BloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k1_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = BloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k1_get(b: &mut Bencher) {
                let mut bf = BloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                (0..1000)
                    .for_each(|n| bf.insert(&n));
                b.iter(|| {
                    (0..1000)
                        .for_each(|n| { bf.contains(&n); });
                })
            }

            #[bench]
            fn k4_get(b: &mut Bencher) {
                let mut bf = BloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                (0..1000)
                    .for_each(|n| bf.insert(&n));
                b.iter(|| {
                    (0..1000)
                        .for_each(|n| { bf.contains(&n); });
                })
            }

            #[bench]
            fn k7_get(b: &mut Bencher) {
                let mut bf = BloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(7));
                (0..1000)
                    .for_each(|n| bf.insert(&n));
                b.iter(|| {
                    (0..1000)
                        .for_each(|n| { bf.contains(&n); });
                })
            }
        }

        mod counting {
            use super::*;

            #[bench]
            fn optimal_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = CountingBloomFilter::<i32, ReHasher<MurmurHasher>>::optimal_new(1000, 0.001);
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k7_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = CountingBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k4_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = CountingBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k1_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = CountingBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k1_get(b: &mut Bencher) {
                let mut bf = CountingBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                (0..1000)
                    .for_each(|n| bf.insert(&n));
                b.iter(|| {
                    (0..1000)
                        .for_each(|n| { bf.contains(&n); });
                })
            }

            #[bench]
            fn k4_get(b: &mut Bencher) {
                let mut bf = CountingBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                (0..1000)
                    .for_each(|n| bf.insert(&n));
                b.iter(|| {
                    (0..1000)
                        .for_each(|n| { bf.contains(&n); });
                })
            }

            #[bench]
            fn k7_get(b: &mut Bencher) {
                let mut bf = CountingBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(7));
                (0..1000)
                    .for_each(|n| bf.insert(&n));
                b.iter(|| {
                    (0..1000)
                        .for_each(|n| { bf.contains(&n); });
                })
            }
        }


        mod wl {
            use super::*;

            #[bench]
            fn optimal_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = WLockBloomFilter::<i32, ReHasher<MurmurHasher>>::optimal_new(1000, 0.001);
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k7_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = WLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k4_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = WLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k1_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = WLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k1_get(b: &mut Bencher) {
                let bf = WLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                (0..1000)
                    .for_each(|n| bf.insert(&n));
                b.iter(|| {
                    (0..1000)
                        .for_each(|n| assert!(bf.contains(&n)));
                })
            }

            #[bench]
            fn k4_get(b: &mut Bencher) {
                let bf = WLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                (0..1000)
                    .for_each(|n| bf.insert(&n));
                b.iter(|| {
                    (0..1000)
                        .for_each(|n| { bf.contains(&n); });
                })
            }

            #[bench]
            fn k7_get(b: &mut Bencher) {
                let bf = WLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(7));
                (0..1000)
                    .for_each(|n| bf.insert(&n));
                b.iter(|| {
                    (0..1000)
                        .for_each(|n| { bf.contains(&n); });
                })
            }
        }

        mod counting_wl {
            use super::*;

            #[bench]
            fn optimal_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = CountingWLockBloomFilter::<i32, ReHasher<MurmurHasher>>::optimal_new(1000, 0.001);
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k7_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = CountingWLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k4_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = CountingWLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k1_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = CountingWLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                    (0..1000)
                        .for_each(|n| bf.insert(&n));
                })
            }

            #[bench]
            fn k1_get(b: &mut Bencher) {
                let bf = CountingWLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                (0..1000)
                    .for_each(|n| bf.insert(&n));
                b.iter(|| {
                    (0..1000)
                        .for_each(|n| assert!(bf.contains(&n)));
                })
            }

            #[bench]
            fn k4_get(b: &mut Bencher) {
                let bf = CountingWLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                (0..1000)
                    .for_each(|n| bf.insert(&n));
                b.iter(|| {
                    (0..1000)
                        .for_each(|n| { bf.contains(&n); });
                })
            }

            #[bench]
            fn k7_get(b: &mut Bencher) {
                let bf = CountingWLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(7));
                (0..1000)
                    .for_each(|n| bf.insert(&n));
                b.iter(|| {
                    (0..1000)
                        .for_each(|n| { bf.contains(&n); });
                })
            }
        }
    }
    
    
    mod single_element {
        use super::*;

        mod normal {
            use super::*;

            #[bench]
            fn optimal_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = BloomFilter::<i32, ReHasher<MurmurHasher>>::optimal_new(1000, 0.001);
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k7_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = BloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k4_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = BloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k1_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = BloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k1_get(b: &mut Bencher) {
                let mut bf = BloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                b.iter(|| {
                    bf.contains(&0);
                })
            }

            #[bench]
            fn k4_get(b: &mut Bencher) {
                let bf = BloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                b.iter(|| {
                    bf.contains(&0);
                })
            }

            #[bench]
            fn k7_get(b: &mut Bencher) {
                let bf = BloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(7));
                b.iter(|| {
                    bf.contains(&0);
                })
            }
        }

        mod counting {
            use super::*;

            #[bench]
            fn optimal_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = CountingBloomFilter::<i32, ReHasher<MurmurHasher>>::optimal_new(1000, 0.001);
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k7_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = CountingBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k4_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = CountingBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k1_insert(b: &mut Bencher) {
                b.iter(|| {
                    let mut bf = CountingBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k1_get(b: &mut Bencher) {
                let bf = CountingBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                b.iter(|| {
                    bf.contains(&0);
                })
            }

            #[bench]
            fn k4_get(b: &mut Bencher) {
                let bf = CountingBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                b.iter(|| {
                    bf.contains(&0);
                })
            }

            #[bench]
            fn k7_get(b: &mut Bencher) {
                let bf = CountingBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(7));
                b.iter(|| {
                    bf.contains(&0);
                })
            }
        }


        mod wl {
            use super::*;

            #[bench]
            fn optimal_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = WLockBloomFilter::<i32, ReHasher<MurmurHasher>>::optimal_new(1000, 0.001);
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k7_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = WLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k4_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = WLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k1_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = WLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k1_get(b: &mut Bencher) {
                let bf = WLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                b.iter(|| {
                    bf.contains(&0)
                })
            }

            #[bench]
            fn k4_get(b: &mut Bencher) {
                let bf = WLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                b.iter(|| {
                    bf.contains(&0);
                })
            }

            #[bench]
            fn k7_get(b: &mut Bencher) {
                let bf = WLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(7));
                b.iter(|| {
                    bf.contains(&0);
                })
            }
        }

        mod counting_wl {
            use super::*;

            #[bench]
            fn optimal_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = CountingWLockBloomFilter::<i32, ReHasher<MurmurHasher>>::optimal_new(1000, 0.001);
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k7_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = CountingWLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k4_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = CountingWLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k1_insert(b: &mut Bencher) {
                b.iter(|| {
                    let bf = CountingWLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                    bf.insert(&0);
                })
            }

            #[bench]
            fn k1_get(b: &mut Bencher) {
                let bf = CountingWLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(1));
                b.iter(|| {
                    bf.contains(&0);
                })
            }

            #[bench]
            fn k4_get(b: &mut Bencher) {
                let bf = CountingWLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(4));
                b.iter(|| {
                    bf.contains(&0);
                })
            }

            #[bench]
            fn k7_get(b: &mut Bencher) {
                let bf = CountingWLockBloomFilter::<i32, ReHasher<MurmurHasher>>::with_rate(1000, 0.001, ReHasher::new(7));
                b.iter(|| {
                    bf.contains(&0);
                })
            }
        }
    }


}