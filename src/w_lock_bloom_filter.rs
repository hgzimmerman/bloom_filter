use crate::hash_to_indicies::HashToIndices;
use crate::hash_to_indicies::K;
use crate::rehasher::ReHasher;
use bit_vec::BitVec;
use std::fmt::Debug;
use std::fmt::Error;
use std::fmt::Formatter;
use std::hash::Hash;
use std::marker::PhantomData;
use std::sync::atomic::AtomicBool;
use std::sync::atomic::Ordering;

/// A variant of a bloom filter with the insert method taking &self, so no mutable reference to the
/// datastructure is needed.
///
/// # Notes
/// This is thread safe because:
/// 1. The size of the backing bitvec is fixed size, and nothing takes any references to it anyway,
/// so the concern about growing+reallocating does not exist.
/// 2. The k values are immutable and act as factories for producing default hashers.
/// If they don't produce hashers in the same state on every invocation, the implementation is broken anyway.
/// 3. Because there is a spinlock on the critical section of the insert operation,
/// no two threads can race and clobber the setting of bits.
///
/// # Warning
/// Do note, that this is a write only lock.
/// That means that if one thread is reading and another is writing,
/// there is no guarantee that the write will finish before the read occurs assuming the
/// operations are dispatched at approximately the same time.
/// This should be Ok for most workloads, although for that brief moment before the write is committed,
/// a false negative is possible.
///
/// If guaranteed absolute ordering is needed, a RwLock<BloomFilter> could be used instead,
/// although that comes with a significant performance cost because the lock would persist
/// while the hashing takes place, which is where the majority of time is spent.
///
pub struct WLockBloomFilter<T, K> {
    pub(crate) bit_vec: *mut BitVec,
    is_writing: AtomicBool,
    type_info: PhantomData<T>,
    pub(crate) k: Box<K>,
}

impl<T, K> Debug for WLockBloomFilter<T, K> {
    fn fmt(&self, f: &mut Formatter) -> Result<(), Error> {
        let mut s = String::new();
        unsafe {
            (*self.bit_vec)
                .iter()
                .for_each(|b| if b { s.push('1') } else { s.push('0') })
        }
        write!(f, "bloom_filter: [{}]", s)
    }
}

unsafe impl<T, K> Send for WLockBloomFilter<T, K>
where
    T: Send,
    K: Sync,
{
}
unsafe impl<T, K> Sync for WLockBloomFilter<T, K>
where
    T: Sync,
    K: Sync,
{
}

impl<T, H> WLockBloomFilter<T, ReHasher<H>> {
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
    /// use bloom_filter::WLockBloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let bf = WLockBloomFilter::<&str, ReHasher<MurmurHasher>>::optimal_new(10000, 0.001);
    /// ```
    pub fn optimal_new(n: usize, p: f64) -> Self {
        let m = crate::optimal_m(n, p);
        let k = crate::optimal_k(n, m);
        WLockBloomFilter {
            bit_vec: Box::into_raw(Box::new(BitVec::from_elem(m, false))),
            is_writing: AtomicBool::new(false),
            type_info: PhantomData,
            k: Box::new(ReHasher::new(k)),
        }
    }
}

impl<T, H> WLockBloomFilter<T, H>
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
    /// use bloom_filter::WLockBloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let bf = WLockBloomFilter::<&str, ReHasher<MurmurHasher>>::with_rate(10000, 0.001, ReHasher::new(1));
    /// ```
    pub fn with_rate(n: usize, p: f64, hashers: H) -> Self {
        let m = crate::m_from_knp(hashers.k(), n, p);
        WLockBloomFilter {
            bit_vec: Box::into_raw(Box::new(BitVec::from_elem(m, false))),
            is_writing: AtomicBool::new(false),
            type_info: PhantomData,
            k: Box::new(hashers),
        }
    }
}

impl<T, K> WLockBloomFilter<T, K>
where
    T: Hash,
    K: HashToIndices,
{
    /// Creates the bloom filter with a given number of bits
    /// and with a multiple-hashing-to-index function.
    /// # Arguments
    /// * `m` - Number of bits for the BloomFilter.
    /// * `hashers` - Hashing to indices structure.
    ///
    /// # Examples
    /// ```
    /// use bloom_filter::WLockBloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let bf = WLockBloomFilter::<&str, ReHasher<MurmurHasher>>::new(100000, ReHasher::new(1));
    /// ```
    pub fn new(m: usize, hashers: K) -> Self {
        WLockBloomFilter {
            bit_vec: Box::into_raw(Box::new(BitVec::from_elem(m, false))),
            is_writing: AtomicBool::new(false),
            type_info: PhantomData,
            k: Box::new(hashers),
        }
    }

    /// Gets the number of bits in the used in the bloom filter.
    pub fn num_bits(&self) -> usize {
        unsafe { self.bit_vec.as_ref().unwrap().len() }
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
    /// use bloom_filter::WLockBloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let bf = WLockBloomFilter::<&str, ReHasher<MurmurHasher>>::new(100000, ReHasher::new(1));
    /// bf.insert(&"hello");
    /// ```
    pub fn insert(&self, value: &T) {
        let indices = self.k.hash_to_indices(value, self.num_bits());
        while self
            .is_writing
            .compare_and_swap(false, true, Ordering::Acquire)
        {
            std::thread::yield_now() // TODO check if this is faster or slower than just spinning. do for various thread counts.
        }
        indices
            .into_iter()
            .for_each(|i| unsafe { self.bit_vec.as_mut().unwrap().set(i, true) });

        // release the lock
        self.is_writing.store(false, Ordering::Release);
    }

    /// Tests to see if the provided value is in the bloom filter.
    /// This will return false positives if the bits that are the result of hashing the value are already set.
    /// Likelihood of false positives will increase as the filter fills up.
    /// This can be mitigated by allocating more bits to the bloom filter, and by increasing the number of hash functions used ('k').
    ///
    ///
    ///
    /// # Arguments
    ///
    /// * `value` - The value to be hashed to create indices into the bloom filter.
    /// These indices will be used to see if the element has been added.
    ///
    /// # Examples
    /// ```
    /// use bloom_filter::WLockBloomFilter;
    /// use bloom_filter::ReHasher;
    /// use murmur3::murmur3_32::MurmurHasher;
    /// let bf = WLockBloomFilter::<&str, ReHasher<MurmurHasher>>::new(100000, ReHasher::new(1));
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
            .all(|i| unsafe { self.bit_vec.as_ref().unwrap()[i] })
    }
}

impl<T, U: K> K for WLockBloomFilter<T, U> {
    fn k(&self) -> usize {
        self.k.k()
    }
}

impl<T, K> Drop for WLockBloomFilter<T, K> {
    fn drop(&mut self) {
        unsafe {
            let _ = Box::from_raw(self.bit_vec);
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    use murmur3::murmur3_32::MurmurHasher;

    #[test]
    fn optimal_constructor() {
        let bf: WLockBloomFilter<&str, ReHasher<MurmurHasher>> =
            WLockBloomFilter::optimal_new(1000, 0.01);
        assert_eq!(bf.num_bits(), 9586);
        assert_eq!(bf.k(), 7)
    }

}
