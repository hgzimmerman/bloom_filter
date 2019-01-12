use crate::hash_to_indicies::HashToIndices;
use crate::hash_to_indicies::K;
use core::hash::BuildHasher;
use core::hash::BuildHasherDefault;
use core::hash::Hash;
use core::hash::Hasher;

// TODO: this file may be better implemented using a macro.

#[derive(Default, Debug)]
pub struct One<H>(BuildHasherDefault<H>);
#[derive(Default, Debug)]
pub struct Two<H1, H2>(BuildHasherDefault<H1>, BuildHasherDefault<H2>);
#[derive(Default, Debug)]
pub struct Three<H1, H2, H3>(
    BuildHasherDefault<H1>,
    BuildHasherDefault<H2>,
    BuildHasherDefault<H3>,
);
#[derive(Default, Debug)]
pub struct Four<H1, H2, H3, H4>(
    BuildHasherDefault<H1>,
    BuildHasherDefault<H2>,
    BuildHasherDefault<H3>,
    BuildHasherDefault<H4>,
);
#[derive(Default, Debug)]
pub struct Five<H1, H2, H3, H4, H5>(
    BuildHasherDefault<H1>,
    BuildHasherDefault<H2>,
    BuildHasherDefault<H3>,
    BuildHasherDefault<H4>,
    BuildHasherDefault<H5>,
);

impl<H> K for BuildHasherDefault<H> {
    fn k(&self) -> usize {
        1
    }
}

impl<H> K for One<H> {
    fn k(&self) -> usize {
        self.0.k()
    }
}
impl<H1, H2> K for Two<H1, H2> {
    fn k(&self) -> usize {
        self.0.k() + self.1.k()
    }
}

impl<H1, H2, H3> K for Three<H1, H2, H3> {
    fn k(&self) -> usize {
        self.0.k() + self.1.k() + self.2.k()
    }
}

impl<H1, H2, H3, H4> K for Four<H1, H2, H3, H4> {
    fn k(&self) -> usize {
        self.0.k() + self.1.k() + self.2.k() + self.3.k()
    }
}

impl<H1, H2, H3, H4, H5> K for Five<H1, H2, H3, H4, H5> {
    fn k(&self) -> usize {
        self.0.k() + self.1.k() + self.2.k() + self.3.k() + self.4.k()
    }
}

impl<H: Hasher + Default> HashToIndices for One<H> {
    fn hash_to_indices<T: Hash>(&self, value: &T, modulus: usize) -> Vec<usize> {
        let mut h = self.0.build_hasher();
        value.hash(&mut h);
        vec![h.finish() as usize % modulus]
    }
}

impl<H: BuildHasher + Default> HashToIndices for H {
    fn hash_to_indices<T: Hash>(&self, value: &T, modulus: usize) -> Vec<usize> {
        let mut h = self.build_hasher();
        value.hash(&mut h);
        vec![h.finish() as usize % modulus]
    }
}

impl<H1, H2> HashToIndices for Two<H1, H2>
where
    H1: Hasher + Default,
    H2: Hasher + Default,
{
    fn hash_to_indices<T: Hash>(&self, value: &T, modulus: usize) -> Vec<usize> {
        let mut h1 = self.0.build_hasher();
        value.hash(&mut h1);

        let mut h2 = self.1.build_hasher();
        value.hash(&mut h2);

        vec![
            h1.finish() as usize % modulus,
            h2.finish() as usize % modulus,
        ]
    }
}
impl<H1, H2, H3> HashToIndices for Three<H1, H2, H3>
where
    H1: Hasher + Default,
    H2: Hasher + Default,
    H3: Hasher + Default,
{
    fn hash_to_indices<T: Hash>(&self, value: &T, modulus: usize) -> Vec<usize> {
        let mut h1 = self.0.build_hasher();
        value.hash(&mut h1);

        let mut h2 = self.1.build_hasher();
        value.hash(&mut h2);

        let mut h3 = self.2.build_hasher();
        value.hash(&mut h3);
        vec![
            h1.finish() as usize % modulus,
            h2.finish() as usize % modulus,
            h3.finish() as usize % modulus,
        ]
    }
}
impl<H1, H2, H3, H4> HashToIndices for Four<H1, H2, H3, H4>
where
    H1: Hasher + Default,
    H2: Hasher + Default,
    H3: Hasher + Default,
    H4: Hasher + Default,
{
    fn hash_to_indices<T: Hash>(&self, value: &T, modulus: usize) -> Vec<usize> {
        let mut h1 = self.0.build_hasher();
        value.hash(&mut h1);

        let mut h2 = self.1.build_hasher();
        value.hash(&mut h2);

        let mut h3 = self.2.build_hasher();
        value.hash(&mut h3);

        let mut h4 = self.3.build_hasher();
        value.hash(&mut h4);
        vec![
            h1.finish() as usize % modulus,
            h2.finish() as usize % modulus,
            h3.finish() as usize % modulus,
            h4.finish() as usize % modulus,
        ]
    }
}
impl<H1, H2, H3, H4, H5> HashToIndices for Five<H1, H2, H3, H4, H5>
where
    H1: Hasher + Default,
    H2: Hasher + Default,
    H3: Hasher + Default,
    H4: Hasher + Default,
    H5: Hasher + Default,
{
    fn hash_to_indices<T: Hash>(&self, value: &T, modulus: usize) -> Vec<usize> {
        let mut h1 = self.0.build_hasher();
        value.hash(&mut h1);

        let mut h2 = self.1.build_hasher();
        value.hash(&mut h2);

        let mut h3 = self.2.build_hasher();
        value.hash(&mut h3);

        let mut h4 = self.3.build_hasher();
        value.hash(&mut h4);

        let mut h5 = self.4.build_hasher();
        value.hash(&mut h5);
        vec![
            h1.finish() as usize % modulus,
            h2.finish() as usize % modulus,
            h3.finish() as usize % modulus,
            h4.finish() as usize % modulus,
            h5.finish() as usize % modulus,
        ]
    }
}
