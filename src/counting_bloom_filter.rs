use std::hash::Hash;
use crate::hash_to_indicies::HashToIndices;
use crate::hash_to_indicies::K as GetK;
use crate::bloom_filter::BloomFilter;
use crate::rehasher::ReHasher;
use crate::hash_to_indicies::K;


/// A bloom filter that counts on each insertion so that it can give a reliable estimate of
/// a false positive rate at its current occupancy level.
pub struct CountingBloomFilter<T,K>{
    /// Backing bloom filter.
    bloom_filter: BloomFilter<T,K>,
    /// The counter that keeps track of the number of elements inserted.
    count: usize
}


impl <T, H> CountingBloomFilter<T, ReHasher<H>> {
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
    /// use bloom_filter::BloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let bf = BloomFilter::<&str, ReHasher<MurmurHasher>>::optimal_new(10000, 0.001);
    /// ```
    pub fn optimal_new(n: usize, p: f64)  -> Self {
        let bloom_filter = BloomFilter::optimal_new(n, p);
        CountingBloomFilter {
            bloom_filter,
            count: 0
        }
    }
}

impl <T, H> CountingBloomFilter<T, H> where H: HashToIndices + GetK {
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
    /// use bloom_filter::BloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let bf = BloomFilter::<&str, ReHasher<MurmurHasher>>::with_rate(10000, 0.001, ReHasher::new(1));
    /// ```
    pub fn with_rate(expected_elements: usize, error_rate: f64, k: H) -> Self {
        CountingBloomFilter {
            bloom_filter: BloomFilter::with_rate(expected_elements, error_rate, k),
            count: 0
        }
    }
}

impl <T,K> CountingBloomFilter<T,K>
where
    T: Hash,
    K: HashToIndices + GetK
{
    /// Creates the bloom filter with a given number of bits and with a multiple-hashing-to-index function.
    ///
    /// # Arguments
    /// * `m` - Number of bits for the BloomFilter.
    /// * `hashers` - Hashing to indices structure.
    ///
    /// # Examples
    /// ```
    /// use bloom_filter::BloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let bf = BloomFilter::<&str, ReHasher<MurmurHasher>>::new(100000, ReHasher::new(1));
    /// ```
    pub fn new(m: usize, hashers: K) -> Self {
        CountingBloomFilter {
            bloom_filter: BloomFilter::new(m, hashers),
            count: 0
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
    /// * `value` - the value to be hashed to create indices into the bloom filter.
    /// These indices will be used to see if the element has been added.
    ///
    /// # Examples
    /// ```
    /// use bloom_filter::CountingBloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let mut bf = CountingBloomFilter::<&str, ReHasher<MurmurHasher>>::new(100000, ReHasher::new(1));
    /// bf.insert(&"hello");
    /// bf.insert(&"there");
    /// ```
    pub fn insert(&mut self, value: &T) {
        self.count += 1;
        self.bloom_filter.insert(value)
    }

    /// Tests to see if the provided value is in the bloom filter.
    /// This will return false positives if the bits that are the result of hashing the value are already set.
    /// Likelihood of false positives will increase as the filter fills up.
    /// This can be mitigated by allocating more bits to the bloom filter, and by increasing the number of hash functions used ('k').
    ///
    ///
    /// # Arguments
    ///
    /// * `value` - the value to be hashed to create indices into the bloom filter.
    /// These indices will be used to see if the element has been added.
    ///
    /// # Examples
    /// ```
    /// use bloom_filter::CountingBloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let mut bf = CountingBloomFilter::<&str, ReHasher<MurmurHasher>>::new(100000, ReHasher::new(1));
    /// bf.insert(&"hello");
    /// bf.insert(&"there");
    /// assert!(bf.contains(&"hello"));
    /// assert!(bf.contains(&"there"));
    /// assert!(!bf.contains(&"not here"));
    /// ```
    pub fn contains(&self, value: &T) -> bool {
        self.bloom_filter.contains(value)
    }

    /// Returns an **estimate** of the current chance that any given lookup will return a false positive.
    pub fn false_positive_chance(&self) -> f64 {
        use crate::false_positive_rate as fpr;
        fpr(self.bloom_filter.k.k(), self.count, self.bloom_filter.num_bits())
    }
}

impl <T, U: K> K for CountingBloomFilter<T,U> {
    fn k(&self) -> usize {
        self.bloom_filter.k.k()
    }
}



#[cfg(test)]
mod tests {
    use super::*;
    use murmur3::murmur3_32::MurmurHasher;
    use crate::hash_numbers::One;

    #[test]
    fn new_false_positive() {
        let cbf: CountingBloomFilter<&str, One<MurmurHasher>> = CountingBloomFilter::new(10_000, One::default());
        assert_eq!(cbf.false_positive_chance(), 0.0);
    }


    #[test]
    fn full_false_positive() {
        let mut cbf: CountingBloomFilter<&str, One<MurmurHasher>> = CountingBloomFilter::new(10, One::default());
        cbf.insert(&"a");
        cbf.insert(&"b");
        cbf.insert(&"c");
        cbf.insert(&"d");
        cbf.insert(&"e");
        cbf.insert(&"f");
        cbf.insert(&"g");
        cbf.insert(&"h");
        cbf.insert(&"i");
        cbf.insert(&"j");
        assert_eq!(cbf.false_positive_chance(), 0.6321205588285577,
                   "This shouldn't be 1.0 as may be expected, because if some _do_collide,\
                then there would be a lower chance of others colliding");
    }
}


