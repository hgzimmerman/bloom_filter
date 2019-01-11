use crate::hash_to_indicies::HashToIndices;
use std::hash::Hasher;
use std::hash::Hash;
use crate::hash_to_indicies::K;
use std::hash::BuildHasherDefault;
use std::hash::BuildHasher;

/// A struct when made to hash a value to indices into the bloom filter,
/// will reuse the same hashbuffer multiple times,
/// seeding the each iteration with the last's buffer state.
pub struct ReHasher<T>{
    k: usize,
    hasher: BuildHasherDefault<T>
}
impl <T> ReHasher<T> {
    pub fn new(k:usize) -> Self {
        ReHasher {
            k,
            hasher: BuildHasherDefault::default()
        }
    }
}

impl <T: Default> Default for ReHasher<T> {
    fn default() -> Self {
        ReHasher {
            /// 4 is a good number, but default() isn't really how this should be constructed
            k: 4,
            hasher: BuildHasherDefault::default()
        }
    }
}

impl <H: Hasher + Default> HashToIndices for ReHasher<H> {
    fn hash_to_indices<T: Hash>(&self, value: &T, modulus: usize) -> Vec<usize> {
        let mut h = self.hasher.build_hasher();
        (0..self.k)
            .map(|_| {
                value.hash(&mut h);
                h.finish() as usize % modulus
            })
            .collect()
    }
}

impl <H> K for ReHasher<H> {
    fn k(&self) -> usize {
        self.k
    }
}


#[cfg(test)]
mod tests {
    use super::*;
    use murmur3::murmur3_32::MurmurHasher;

    #[test]
    fn re_hasher_different_outputs_hello() {
        let rehasher: ReHasher<MurmurHasher> = ReHasher::new(3);
        let indices = rehasher.hash_to_indices(&"hello", 1000);
        assert_eq!(indices, vec![26, 16, 434]);
        let rehasher: ReHasher<MurmurHasher> = ReHasher::new(4);
        let indices = rehasher.hash_to_indices(&"hello", 1000);
        assert_eq!(indices, vec![26, 16, 434, 927])
    }

    #[test]
    fn re_hasher_different_outputs_there() {
        let rehasher: ReHasher<MurmurHasher> = ReHasher::new(4);
        let indices = rehasher.hash_to_indices(&"there", 1000);
        assert_eq!(indices, vec![774, 836, 27, 178])
    }
}
