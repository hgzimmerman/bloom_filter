use crate::hash_to_indicies::HashToIndices;
use crate::hash_to_indicies::K;
use crate::rehasher::ReHasher;
use bit_vec::BitVec;
use std::fmt::Debug;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::Hash;
use std::marker::PhantomData;

/// A probabilistic datastructure that can quickly tell with complete accuracy if an element has _not_ been
/// added to itself, but allows false positives when determining if an element has been added.
/// This false positive rate is influenced by the number of hash functions used and the size of the backing bit vector,
/// as well as the number of entries that have been recorded.
pub struct BloomFilter<T, K> {
    /// The backing bit vector.
    pub(crate) bit_vec: BitVec,
    /// The type information of what the bitvector will accept as input.
    type_info: PhantomData<T>,
    /// The generic hashing structure.
    pub(crate) k: Box<K>,
}

impl<T, K> Debug for BloomFilter<T, K> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut s = String::new();
        self.bit_vec
            .iter()
            .for_each(|b| if b { s.push('1') } else { s.push('0') });
        write!(f, "bit_vec: [{}]", s)
    }
}

impl<T, H> BloomFilter<T, ReHasher<H>> {
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
    pub fn optimal_new(n: usize, p: f64) -> Self {
        let m = crate::optimal_m(n, p);
        let k = crate::optimal_k(n, m);
        BloomFilter {
            bit_vec: BitVec::from_elem(m, false),
            type_info: PhantomData,
            k: Box::new(ReHasher::new(k)),
        }
    }
}

impl<T, H> BloomFilter<T, H>
where
    H: HashToIndices + K,
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
    /// use bloom_filter::BloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let bf = BloomFilter::<&str, ReHasher<MurmurHasher>>::with_rate(10000, 0.001, ReHasher::new(1));
    /// ```
    pub fn with_rate(n: usize, p: f64, hashers: H) -> Self {
        let m = crate::m_from_knp(hashers.k(), n, p);
        BloomFilter {
            bit_vec: BitVec::from_elem(m, false),
            type_info: PhantomData,
            k: Box::new(hashers),
        }
    }
}

impl<T, K> BloomFilter<T, K>
where
    T: Hash,
    K: HashToIndices,
{
    /// Creates the bloom filter with a given number of bits
    /// and with a multiple-hashing-to-index function.
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
        BloomFilter {
            bit_vec: BitVec::from_elem(m, false),
            type_info: PhantomData,
            k: Box::new(hashers),
        }
    }

    /// Gets the number of bits in the used in the bloom filter.
    pub fn num_bits(&self) -> usize {
        self.bit_vec.len()
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
    /// use bloom_filter::BloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let mut bf = BloomFilter::<&str, ReHasher<MurmurHasher>>::new(100000, ReHasher::new(1));
    /// bf.insert(&"hello");
    /// bf.insert(&"there");
    /// ```
    pub fn insert(&mut self, value: &T) {
        self.k
            .hash_to_indices(value, self.num_bits())
            .into_iter()
            .for_each(|i| self.bit_vec.set(i, true));
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
    /// use bloom_filter::BloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let mut bf = BloomFilter::<&str, ReHasher<MurmurHasher>>::new(100000, ReHasher::new(1));
    /// bf.insert(&"hello");
    /// bf.insert(&"there");
    /// assert!(bf.contains(&"hello"));
    /// assert!(bf.contains(&"there"));
    /// assert!(!bf.contains(&"not here"));
    /// ```
    pub fn contains(&self, value: &T) -> bool {
        self.k
            .hash_to_indices(value, self.num_bits())
            .into_iter()
            .all(|i| self.bit_vec[i])
    }
}

impl<T, U: K> K for BloomFilter<T, U> {
    fn k(&self) -> usize {
        self.k.k()
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::hash_numbers::One;
    use crate::hash_numbers::Two;
    use murmur3::murmur3_32::MurmurHasher;

    use crate::false_positive_rate;
    use crate::rehasher::ReHasher;
    use hashers::fnv::FNV1aHasher32;

    #[test]
    fn t_false_positive_rate() {
        let x = false_positive_rate(4, 10_000, 100_000);
        assert!(x < 0.012)
    }

    #[test]
    fn k_1_contains() {
        let mut bf: BloomFilter<&str, ReHasher<MurmurHasher>> =
            BloomFilter::new(100000, ReHasher::new(3));
        bf.insert(&"hello");
        assert!(bf.contains(&"hello"))
    }

    #[test]
    fn k_1_does_not_contain() {
        let mut bf: BloomFilter<&str, One<MurmurHasher>> = BloomFilter::new(100000, One::default());
        bf.insert(&"hello");
        assert!(!bf.contains(&"there"))
    }

    #[test]
    fn k_1_false_positives_are_possible() {
        let mut bf: BloomFilter<&str, One<MurmurHasher>> = BloomFilter::new(5, One::default());
        bf.insert(&"a");
        assert!(
            bf.contains(&"l"),
            "With a murmur hasher, a and l should resolve to the same index"
        )
    }

    #[test]
    fn false_positives_can_be_avoided_with_more_k() {
        let mut bf: BloomFilter<&str, One<MurmurHasher>> = BloomFilter::new(5, One::default());
        bf.insert(&"a");
        assert!(
            bf.contains(&"l"),
            "With a murmur hasher, a and l should resolve to the same index"
        );
        let mut bf: BloomFilter<&str, Two<MurmurHasher, FNV1aHasher32>> =
            BloomFilter::new(5, Two::default());
        bf.insert(&"a");
        assert!(
            !bf.contains(&"l"),
            "With two hashers, a and l should have one index be the same,\
             but the other is allowed to be different, permitting avoidance of the false positive"
        )
    }

    #[test]
    fn optimal_constructor() {
        let bf: BloomFilter<&str, ReHasher<FNV1aHasher32>> = BloomFilter::optimal_new(1000, 0.01);
        assert_eq!(bf.num_bits(), 9586);
        assert_eq!(bf.k(), 7)
    }

    #[test]
    fn with_rate_constructor() {
        let bf: BloomFilter<&str, ReHasher<MurmurHasher>> =
            BloomFilter::with_rate(1000, 0.0001, ReHasher::new(4));
        assert_eq!(bf.num_bits(), 37_964)
    }
}
