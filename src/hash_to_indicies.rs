use std::hash::Hash;

pub trait HashToIndices: Default {
    #[inline]
    fn hash_to_indices<T: Hash>(&self, value:  &T, modulus: usize) -> Vec<usize>;
}

pub trait K {
    #[inline]
    fn k(&self) -> usize;
}