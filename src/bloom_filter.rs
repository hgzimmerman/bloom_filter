use bit_vec::BitVec;
use std::marker::PhantomData;
use std::hash::Hash;
use crate::hash_to_indicies::HashToIndices;
use std::fmt::Debug;
use std::fmt::Formatter;
use std::fmt::Error;
use crate::rehasher::ReHasher;


/// A probabilistic datastructure that can quickly tell with complete accuracy if an element has _not_ been
/// added to itself, but allows false positives when determining if an element has been added.
/// This false positive rate is influenced by the number of hash functions used and the size of the backing bit vector.
pub struct BloomFilter<T, K>(pub(crate) BitVec, PhantomData<T>, pub(crate) Box<K>);

impl <T, K> Debug for BloomFilter<T, K> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut s = String::new();
        self.0.iter().for_each(|b| if b {s.push('1')} else {s.push('0')});
        write!(f, "bit_vec: [{}]", s)
    }
}


impl <T, H> BloomFilter<T, ReHasher<H>> {
    /// n: number of expected elements.
    /// p: false positive rate desired at `n`.
    pub fn optimal_new(n: usize, p: f64)  -> Self {
        let m = crate::needed_size(n, p);
        let k = crate::optimal_k(n, m);
        BloomFilter(BitVec::from_elem(m, false), PhantomData, Box::new(ReHasher::new(k)))
    }
}

impl <T, K> BloomFilter<T, K>
    where
        T: Hash,
        K: HashToIndices
{
    /// Creates the bloom filter with a given number of bits and with a multiple-hashing-to-index function.
    pub fn new(num_bits: usize, k: K) -> Self {
        BloomFilter(BitVec::from_elem(num_bits, false), PhantomData, Box::new(k))
    }

    /// Gets the number of bits in the used in the bloom filter.
    pub fn num_bits(&self) -> usize {
        self.0.len()
    }

    /// Takes multiple hashes of the provided value, takes the hashes modulo the number of bits
    /// (converting them to indexes) and sets those bits in the backing bitvec to 1.
    /// If a bit is already set to 1, then there will be a collision with that particular bit.
    /// This won't result in an actual false positive when `contains()` is called unless `k` is 1.
    /// A higher `k` value requires that `k` hash-indices need to collide for an actual false positive to occur.
    /// The drawback of a higher k is that it takes longer for each insert/lookup and that the filter will fill up faster.
    pub fn insert(&mut self, value: &T) {
        self.2
            .hash_to_indices(value, self.num_bits())
            .into_iter()
            .for_each(|i| self.0.set(i, true));
    }

    /// Tests to see if the provided value is in the bloom filter.
    /// This will return false positives if the bits that are the result of hashing the value are already set.
    /// Likelihood of false positives will increase as the filter fills up.
    /// This can be mitigated by allocating more bits to the bloom filter, and by increasing the number of hash functions used ('k').
    ///
    pub fn contains(&self, value: &T) -> bool {
        self.2
            .hash_to_indices(value, self.num_bits())
            .into_iter()
            .all(|i| self.0[i])
    }
}




#[cfg(test)]
mod tests {
    use super::*;
    use murmur3::murmur3_32::MurmurHasher;
    use crate::hash_numbers::One;
    use crate::hash_numbers::Two;

    use hashers::fnv::FNV1aHasher32;
    use crate::false_positive_rate;
    use crate::rehasher::ReHasher;

    #[test]
    fn t_false_positive_rate() {
        let x = false_positive_rate(4, 10_000, 100_000);
        assert!(x < 0.012)
    }


    #[test]
    fn k_1_contains() {
        let mut bf: BloomFilter<&str, ReHasher<MurmurHasher>> = BloomFilter::new(100000, ReHasher::new(3));
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
        assert!(bf.contains(&"l"), "With a murmur hasher, a and l should resolve to the same index")
    }

    #[test]
    fn false_positives_can_be_avoided_with_more_k() {
        let mut bf: BloomFilter<&str, Two<MurmurHasher, FNV1aHasher32>> = BloomFilter::new(5, Two::default());
        bf.insert(&"a");
        assert!(!bf.contains(&"l"), "With two hashers, a and l should have one index be the same,\
         but the other is allowed to be different, permitting avoidance of the false positive")
    }

}