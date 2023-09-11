/*
 * Copyright © 2023 Anand Beh
 *
 * Licensed under the Apache License, Version 2.0 (the "License");
 * you may not use this file except in compliance with the License.
 * You may obtain a copy of the License at
 *
 *     http://www.apache.org/licenses/LICENSE-2.0
 *
 * Unless required by applicable law or agreed to in writing, software
 * distributed under the License is distributed on an "AS IS" BASIS,
 * WITHOUT WARRANTIES OR CONDITIONS OF ANY KIND, either express or implied.
 * See the License for the specific language governing permissions and
 * limitations under the License.
 */

#![forbid(unsafe_code)]

//!
//! This crate is dedicated to the sole purpose of hashing collections of elements
//! in an iteration order-independent fashion. For example, it can be used to hash
//! a `HashMap` or `HashSet`.
//!
//! Documentation referring to a "collection" means any type `C` where `&C: IntoIterator`
//!

use std::collections::hash_map::DefaultHasher;
use std::collections::{HashMap, HashSet};
use std::hash::{BuildHasher, Hash, Hasher};
use std::marker::PhantomData;
use std::num::Wrapping;
use std::ops::{Deref, DerefMut};

///
/// Implements hashing by summing the hashes of each element. A new [`DefaultHasher`]
/// is created for each element, its result added to the total calculation.
///
pub fn hash_by_summing_hashes<C, H>(collection: &C, state: &mut H)
where
    for<'c> &'c C: IntoIterator,
    for<'c> <&'c C as IntoIterator>::Item: Hash,
    H: Hasher,
{
    hash_by_summing_hashes_with::<C, H, UseDefaultHasher>(collection, state)
}

///
/// The main function implementing hashing by summing the hashes of each element,
/// with a means of specifying which kind of hasher is created per element via the `BH`
/// parameter.
///
pub fn hash_by_summing_hashes_with<C, H, BH>(collection: &C, state: &mut H)
where
    for<'c> &'c C: IntoIterator,
    for<'c> <&'c C as IntoIterator>::Item: Hash,
    BH: BuildHasherFromFriend<C>,
    H: Hasher,
{
    let mut sum = Wrapping::default();
    for value in collection {
        let mut hasher = BH::build_hasher_from(collection);
        Hash::hash(&value, &mut hasher);
        sum += hasher.finish();
    }
    state.write_u64(sum.0);
}

///
/// Adds hashing to any collection according to the hash of each element, but without
/// respecting iteration order. Instantly usable with `HashMap` or `HashSet`. `Deref` and
/// `DerefMut` provide access to the wrapped type. To create, use the `new` method or `From`.
///
/// ```rust
/// # use std::collections::HashMap;
/// use hash_that_set::SumHashes;
///
/// let my_map: HashMap<i8, String> = HashMap::new();
/// let mut my_map = SumHashes::new(my_map);
///
/// my_map.insert(2, String::from("hello"));
/// ```
///
/// This may be used with any collection, although it requires the wrapped collection to implement
/// [`ProvidesHasher`].
///
/// The layout of this struct is guaranteed to be the same as the wrapped collection. This means
/// it is possible to transmute references; however, [`hash_by_summing_hashes`] is usually a better
/// option than relying on `unsafe`.
///
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[repr(transparent)]
pub struct SumHashes<C: ProvidesHasher>(SumHashesAnyCollection<C, UseProvidedHasher<C>>);

///
/// Adds hashing to any collection according to the hash of each element, but without
/// respecting iteration order. Always usable with any collection, via the default hasher.
/// `Deref` and `DerefMut` provide access to the wrapped type.
///
/// **Do not use this wrapper with an ordered collection**. The wrapper does not change equality
/// semantics; it affects hashing only.
///
/// The layout of this struct is guaranteed to be the same as the wrapped collection. This means
/// it is possible to transmute references; however, [`hash_by_summing_hashes_with`] is usually a
/// better option than relying on `unsafe`.
///
#[derive(Clone, Debug, Default, Eq, PartialEq)]
#[repr(transparent)]
pub struct SumHashesAnyCollection<C, H = UseDefaultHasher>(C, PhantomData<H>);

impl<C: ProvidesHasher> From<C> for SumHashes<C> {
    /// Creates the wrapper
    #[inline]
    fn from(value: C) -> Self {
        Self(SumHashesAnyCollection::from(value))
    }
}

impl<C: ProvidesHasher> SumHashes<C> {
    /// Creates the wrapper
    #[inline]
    pub fn new(value: C) -> Self {
        Self::from(value)
    }

    /// Destructures into the inner collection
    #[inline]
    pub fn into_inner(self) -> C {
        self.0 .0
    }
}

impl<C, H> From<C> for SumHashesAnyCollection<C, H> {
    /// Creates the wrapper
    #[inline]
    fn from(value: C) -> Self {
        Self(value, PhantomData)
    }
}

impl<C, H> SumHashesAnyCollection<C, H> {
    /// Creates the wrapper
    #[inline]
    pub fn new(value: C) -> Self {
        Self::from(value)
    }

    /// Destructures into the inner collection
    #[inline]
    pub fn into_inner(self) -> C {
        self.0
    }
}

/// Like the standard library's [`BuildHasher`], but takes the hashing implementation from a peer
pub trait BuildHasherFromFriend<F> {
    /// The type of the hasher that will be created
    type Hasher: Hasher;

    /// Creates a hashing implementation
    fn build_hasher_from<'f>(friend: &'f F) -> Self::Hasher
    where
        Self::Hasher: 'f;
}

/// Implementation of [`BuildHasherFromFriend`] which uses the default hasher, always
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UseDefaultHasher(());

impl<F> BuildHasherFromFriend<F> for UseDefaultHasher {
    type Hasher = DefaultHasher;

    fn build_hasher_from<'f>(_: &'f F) -> Self::Hasher
    where
        Self::Hasher: 'f,
    {
        DefaultHasher::new()
    }
}

/// Implementation of [`BuildHasherFromFriend`] which requires that the peer provides the hasher,
/// i.e. that [`ProvidesHasher`] is implemented for the peer object
#[derive(Clone, Copy, Debug, Default, Eq, PartialEq)]
pub struct UseProvidedHasher<C: ProvidesHasher>(PhantomData<C>);

impl<C: ProvidesHasher> BuildHasherFromFriend<C> for UseProvidedHasher<C> {
    type Hasher = <<C as ProvidesHasher>::Hasher as BuildHasher>::Hasher;

    fn build_hasher_from<'f>(friend: &'f C) -> Self::Hasher
    where
        Self::Hasher: 'f,
    {
        ProvidesHasher::hasher(friend).build_hasher()
    }
}

///
/// Trait for types which provide a hashing implementation. This is automatically implemented
/// for `HashMap` and `HashSet`. It allows the wrapper [`SumHashes`] to use the same
/// hashing implementation for elements as is used for the whole hash result.
///
/// PRs are welcome to add features for collections from other crates which yield their hashers.
///
pub trait ProvidesHasher {
    /// The type of the hashing implementation
    type Hasher: BuildHasher;

    /// Returns a reference to the used hasher
    fn hasher(&self) -> &Self::Hasher;
}

impl<K, V, S> ProvidesHasher for HashMap<K, V, S>
where
    S: BuildHasher,
{
    type Hasher = S;

    fn hasher(&self) -> &Self::Hasher {
        HashMap::hasher(self)
    }
}

impl<O, S> ProvidesHasher for HashSet<O, S>
where
    S: BuildHasher,
{
    type Hasher = S;

    fn hasher(&self) -> &Self::Hasher {
        HashSet::hasher(self)
    }
}

impl<C: ProvidesHasher> Hash for SumHashes<C>
where
    for<'c> &'c C: IntoIterator,
    for<'c> <&'c C as IntoIterator>::Item: Hash,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        Hash::hash(&self.0, state)
    }
}

impl<C, BH> Hash for SumHashesAnyCollection<C, BH>
where
    for<'c> &'c C: IntoIterator,
    for<'c> <&'c C as IntoIterator>::Item: Hash,
    BH: BuildHasherFromFriend<C>,
{
    fn hash<H: Hasher>(&self, state: &mut H) {
        hash_by_summing_hashes_with::<C, H, BH>(&self.0, state)
    }
}

impl<C: ProvidesHasher + IntoIterator> IntoIterator for SumHashes<C> {
    type Item = <C as IntoIterator>::Item;
    type IntoIter = <C as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0 .0.into_iter()
    }
}

impl<C: IntoIterator> IntoIterator for SumHashesAnyCollection<C> {
    type Item = <C as IntoIterator>::Item;
    type IntoIter = <C as IntoIterator>::IntoIter;

    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl<C: ProvidesHasher> Deref for SumHashes<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0 .0
    }
}

impl<C: ProvidesHasher> DerefMut for SumHashes<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0 .0
    }
}

impl<C> Deref for SumHashesAnyCollection<C> {
    type Target = C;

    fn deref(&self) -> &Self::Target {
        &self.0
    }
}

impl<C> DerefMut for SumHashesAnyCollection<C> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.0
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn maps_and_sets_impl_hash() {
        static_assertions::assert_impl_all!(SumHashes<HashMap<i8, &str>>: Hash);
        static_assertions::assert_impl_all!(SumHashes<HashSet<i8>>: Hash);
    }

    #[test]
    fn any_collection_impl_hash() {
        // In general, using an array with our library is a contractual violation
        // However, this is a test, so it doesn't matter
        static_assertions::assert_impl_all!(SumHashesAnyCollection<[&str; 5]>: Hash);
    }

    #[test]
    fn default_if_possible() {
        let _: SumHashes<HashMap<i8, &str>> = Default::default();
    }

    #[test]
    fn same_elements_produce_identical_hash() {
        // To simulate differences in iteration order, use sorting and different data structure
        let unsorted = vec![(4, ""), (1, "hi"), (-3, "hello"), (20, "good bye")];
        let mut sorted = Vec::from_iter(unsorted.clone());
        sorted.sort();
        let map: HashMap<i8, &str> = unsorted.iter().cloned().collect();
        let sorted_map: HashMap<i8, &str> = sorted.iter().cloned().collect();
        let set: HashSet<(i8, &str)> = unsorted.iter().cloned().collect();
        let sorted_set: HashSet<(i8, &str)> = sorted.iter().cloned().collect();

        macro_rules! gen_hash {
            ($var:ident) => {{
                let wrapper = SumHashesAnyCollection::<_, UseDefaultHasher>::from($var);
                let mut hasher = DefaultHasher::new();
                Hash::hash(&wrapper, &mut hasher);
                hasher.finish()
            }};
        }
        let all_hashes = [
            gen_hash!(unsorted),
            gen_hash!(sorted),
            gen_hash!(map),
            gen_hash!(sorted_map),
            gen_hash!(set),
            gen_hash!(sorted_set),
        ];
        let hash = all_hashes[0];
        for other in all_hashes {
            assert_eq!(hash, other);
        }
    }
}
