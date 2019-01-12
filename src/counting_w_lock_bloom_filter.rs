use crate::hash_to_indicies::HashToIndices;
use crate::hash_to_indicies::K as GetK;
use crate::hash_to_indicies::K;
use crate::rehasher::ReHasher;
use crate::w_lock_bloom_filter::WLockBloomFilter;
use core::hash::Hash;
use core::sync::atomic::AtomicUsize;
use core::sync::atomic::Ordering;

/// A bloom filter with a spinlock permitting writes and an atomic counter to allow
/// assessing the percentage chance of a false positive.
pub struct CountingWLockBloomFilter<T, K> {
    bloom_filter: WLockBloomFilter<T, K>,
    count: AtomicUsize,
}

impl<T, H> CountingWLockBloomFilter<T, ReHasher<H>> {
    /// Constructs a new BloomFilter with an optimal ratio of m and k,
    /// derived from n and p inputs.
    ///
    /// # Arguments
    ///
    /// * `n` - Number of expected elements to be inserted into the set.
    /// * `p` - False positive rate.
    ///
    /// # Examples
    /// ```
    /// use bloom_filter::CountingWLockBloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let bf = CountingWLockBloomFilter::<&str, ReHasher<MurmurHasher>>::optimal_new(10000, 0.001);
    /// ```
    pub fn optimal_new(n: usize, p: f64) -> Self {
        let bloom_filter = WLockBloomFilter::optimal_new(n, p);
        CountingWLockBloomFilter {
            bloom_filter,
            count: AtomicUsize::new(0),
        }
    }
}

impl<T, H> CountingWLockBloomFilter<T, H>
where
    H: HashToIndices + GetK,
{
    /// Given a fixed size `k`, and an expected number of elements (`n`),
    /// initialize the bloom filter with a computed `m` value to achieve the required error rate.
    ///
    /// # Arguments
    ///
    /// * `n` - Number of expected elements to be inserted into the set.
    /// * `p` - False positive rate.
    /// * `hashers` - Hashing to indicies struct. `k` can be acquired from this.
    ///
    /// # Remarks
    /// Because memory size may not be a premium, and hashing may be computationally expensive,
    /// this function creates a BloomFilter with a fixed number of hashers,
    /// while taking up more memory space than would otherwise be optimal.
    ///
    /// Because the insert and checking time scales with `k`, not with `m`,
    /// `m` can be increased to trade space efficiency for speed.
    ///
    /// # Examples
    /// ```
    /// use bloom_filter::CountingWLockBloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let bf = CountingWLockBloomFilter::<&str, ReHasher<MurmurHasher>>::with_rate(10000, 0.001, ReHasher::new(1));
    /// ```
    pub fn with_rate(n: usize, p: f64, hashers: H) -> Self {
        CountingWLockBloomFilter {
            bloom_filter: WLockBloomFilter::with_rate(n, p, hashers),
            count: AtomicUsize::new(0),
        }
    }
}

impl<T, K> CountingWLockBloomFilter<T, K>
where
    T: Hash,
    K: HashToIndices + GetK,
{
    /// Creates the bloom filter with a given number of bits
    /// and with a multiple-hashing-to-index function.
    /// # Arguments
    /// * `m` - Number of bits for the BloomFilter.
    /// * `hashers` - Hashing to indices structure.
    ///
    /// # Examples
    /// ```
    /// use bloom_filter::CountingWLockBloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let bf = CountingWLockBloomFilter::<&str, ReHasher<MurmurHasher>>::new(100000, ReHasher::new(1));
    /// ```
    pub fn new(m: usize, hashers: K) -> Self {
        CountingWLockBloomFilter {
            bloom_filter: WLockBloomFilter::new(m, hashers),
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
    ///
    /// # Arguments
    ///
    /// * `value` - The value to be hashed to create indices into the bloom filter.
    ///
    /// # Examples
    /// ```
    /// use bloom_filter::CountingWLockBloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let bf = CountingWLockBloomFilter::<&str, ReHasher<MurmurHasher>>::new(100000, ReHasher::new(1));
    /// bf.insert(&"hello");
    /// ```
    pub fn insert(&self, value: &T) {
        self.bloom_filter.insert(value);
        self.count.fetch_add(1, Ordering::Acquire);
    }

    /// Tests to see if the provided value is in the bloom filter.
    /// This will return false positives if the bits that are the result of hashing the value are already set.
    /// Likelihood of false positives will increase as the filter fills up.
    /// This can be mitigated by allocating more bits to the bloom filter, and by increasing the number of hash functions used ('k').
    ///
    /// # Arguments
    ///
    /// * `value` - The value to be hashed to create indices into the bloom filter.
    /// These indices will be used to see if the element has been added.
    ///
    /// # Examples
    /// ```
    /// use bloom_filter::CountingWLockBloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let bf = CountingWLockBloomFilter::<&str, ReHasher<MurmurHasher>>::new(100000, ReHasher::new(1));
    /// bf.insert(&"hello");
    /// bf.insert(&"there");
    /// assert!(bf.contains(&"hello"));
    /// assert!(bf.contains(&"there"));
    /// assert!(!bf.contains(&"not here"));
    /// ```
    pub fn contains(&self, value: &T) -> bool {
        self.bloom_filter.contains(value)
    }

    /// Returns the current chance that any given lookup will return a false positive.
    ///
    /// # Note
    /// The accuracy of the false positive chance is correlated with how evenly distributed the chosen hashing method is.
    /// As it stands, most users will choose a fast hashing method that may not necessarily have a perfectly distributed hash output.
    /// Because of this, the actual incidence of false positives may be higher than indicated here.
    ///
    /// # Examples
    /// ```
    /// use bloom_filter::CountingWLockBloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let bf = CountingWLockBloomFilter::<&str, ReHasher<MurmurHasher>>::new(100, ReHasher::new(1));
    /// assert_eq!(bf.false_positive_chance(), 0.0);
    /// bf.insert(&"hello");
    /// assert_eq!(bf.false_positive_chance(), 0.009950166250831893 );
    pub fn false_positive_chance(&self) -> f64 {
        use crate::false_positive_rate as fpr;
        fpr(
            self.bloom_filter.k.k(),
            self.count.load(Ordering::Relaxed),
            self.bloom_filter.num_bits(),
        )
    }
}

impl<T, U: K> K for CountingWLockBloomFilter<T, U> {
    fn k(&self) -> usize {
        self.bloom_filter.k.k()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use murmur3::murmur3_32::MurmurHasher;

    #[test]
    fn optimal_constructor() {
        let bf: CountingWLockBloomFilter<&str, ReHasher<MurmurHasher>> =
            CountingWLockBloomFilter::optimal_new(1000, 0.01);
        assert_eq!(bf.num_bits(), 9586);
        assert_eq!(bf.k(), 7)
    }

}
