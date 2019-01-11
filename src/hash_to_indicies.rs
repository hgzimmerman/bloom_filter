use std::hash::Hash;

pub trait HashToIndices: Default {
    /// When called, will return a vector of indices.
    /// The length of the vector should correspond to how many hash operations were performed.
    ///
    /// # Note
    /// The indices should be generated by hashing the value and taking the modulus of the
    /// resulting hash value.
    /// The usizes in the returned vector are only considered indices because they are constrained to
    /// the size of the BitVec used in the BloomFilter.
    fn hash_to_indices<T: Hash>(&self, value: &T, modulus: usize) -> Vec<usize>;
}

pub trait K {
    /// A function that returns the k value for a given implementor.
    /// The `k` value should correspond to how many indices will be produced
    /// when hash_to_indicies() is called on it.
    fn k(&self) -> usize;
}
