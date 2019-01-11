use std::hash::Hash;
use crate::hash_to_indicies::HashToIndices;
use std::sync::atomic::Ordering;
use crate::hash_to_indicies::K as GetK;
use crate::w_lock_bloom_filter::WLockBloomFilter;
use std::sync::atomic::AtomicUsize;
use crate::rehasher::ReHasher;


/// A bloom filter with a spinlock permitting writes and an atomic counter to allow
/// assessing the percentage chance of a false positive.
pub struct CountingWLockBloomFilter<T, K>{
    bloom_filter: WLockBloomFilter<T,K>,
    count: AtomicUsize,
}

impl <T, H> CountingWLockBloomFilter<T, ReHasher<H>> {
    /// n: number of expected elements.
    /// p: false positive rate desired at `n`.
    pub fn optimal_new(n: usize, p: f64)  -> Self {
        let bloom_filter = WLockBloomFilter::optimal_new(n, p);
        CountingWLockBloomFilter {
            bloom_filter,
            count: AtomicUsize::new(0)
        }
    }
}

impl <T, K> CountingWLockBloomFilter<T, K>
    where
        T: Hash,
        K: HashToIndices + GetK
{
    /// Creates the bloom filter with a given number of bits and with a multiple-hashing-to-index function.
    pub fn new(num_bits: usize, k: K) -> Self {
        CountingWLockBloomFilter {
            bloom_filter: WLockBloomFilter::new(num_bits, k),
            count: AtomicUsize::new(0),
        }
    }

    /// Gets the number of bits in the used in the bloom filter.
    pub fn num_bits(&self) -> usize {
        self.bloom_filter.num_bits()
    }

    /// Takes multiple hashes of the provided value, takes the hashes modulo the number of bits
    /// (converting them to indexes) and sets those bits in the backing bitvec to 1.
    /// If a bit is already set to 1, then there will be a collision with that particular bit.
    /// This won't result in an actual false positive when `contains()` is called unless `k` is 1.
    /// A higher `k` value requires that `k` hash-indices need to collide for an actual false positive to occur.
    /// The drawback of a higher k is that it takes longer for each insert/lookup and that the filter will fill up faster.
    pub fn insert(&self, value: &T) {
        self.bloom_filter.insert(value);
        self.count.fetch_add(1, Ordering::Acquire);
    }

    /// Tests to see if the provided value is in the bloom filter.
    /// This will return false positives if the bits that are the result of hashing the value are already set.
    /// Likelihood of false positives will increase as the filter fills up.
    /// This can be mitigated by allocating more bits to the bloom filter, and by increasing the number of hash functions used ('k').
    ///
    pub fn contains(&self, value: &T) -> bool {
        self.bloom_filter.contains(value)
    }

    /// Returns an **estimate** of the current chance that any given lookup will return a false positive.
    pub fn false_positive_chance(&self) -> f64 {
        use crate::false_positive_rate as fpr;
        fpr(self.bloom_filter.k.k(), self.count.load(Ordering::Relaxed), self.bloom_filter.num_bits())
    }
}
