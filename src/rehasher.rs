use std::marker::PhantomData;
use crate::hash_numbers::HashToIndices;
use std::hash::Hasher;
use std::hash::Hash;
use crate::hash_numbers::K;

pub struct ReHasher<T>{
    k: usize,
    hasher: PhantomData<T>
}
impl <T> ReHasher<T> {
    pub fn new(k:usize) -> Self {
        ReHasher {
            k,
            hasher: PhantomData
        }
    }
}

impl <T: Default> Default for ReHasher<T> {
    fn default() -> Self {
        ReHasher {
            k: 4,
            hasher: PhantomData
        }
    }
}

impl <H: Hasher + Default> HashToIndices for ReHasher<H> {
    fn hash_to_indices<T: Hash>(&self, value: &T, modulus: usize) -> Vec<usize> {
        let mut h = H::default();
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
        assert_eq!(indices, vec![26, 16, 434])
    }
    #[test]
    fn re_hasher_different_outputs_there() {
        let rehasher: ReHasher<MurmurHasher> = ReHasher::new(4);
        let indices = rehasher.hash_to_indices(&"there", 1000);
        assert_eq!(indices, vec![774, 836, 27, 178])
    }
}
