use std::hash::Hash;
use crate::hash_to_indicies::HashToIndices;
use crate::hash_to_indicies::K as GetK;
use crate::bloom_filter::BloomFilter;
use crate::rehasher::ReHasher;


/// A bloom filter that counts on each insertion so that it can give a reliable
pub struct CountingBloomFilter<T,K>{
    bloom_filter: BloomFilter<T,K>,
    count: usize
}


impl <T, H> CountingBloomFilter<T, ReHasher<H>> {
    /// n: number of expected elements.
    /// p: false positive rate desired at `n`.
    pub fn optimal_new(n: usize, p: f64)  -> Self {
        let bloom_filter = BloomFilter::optimal_new(n, p);
        CountingBloomFilter {
            bloom_filter,
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
    pub fn new(num_bits: usize, k: K) -> Self {
        CountingBloomFilter {
            bloom_filter: BloomFilter::new(num_bits, k),
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
    pub fn insert(&mut self, value: &T) {
        self.count += 1;
        self.bloom_filter.insert(value)
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
        fpr(self.bloom_filter.2.k(), self.count, self.bloom_filter.num_bits())
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


