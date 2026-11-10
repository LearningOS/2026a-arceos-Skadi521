//! Collections, mirroring `std::collections`.
//!
//! [`alloc::collections`] only offers tree-based containers, so a hash map is
//! provided here on top of `hashbrown`.
//!
//! The hasher is a deterministic FNV-1a. `std` uses a randomly seeded SipHash to
//! defend against hash-flooding from untrusted input, which requires an entropy
//! source that a bare-metal kernel does not have; keys here are produced by the
//! kernel itself, so a fixed seed is sufficient and keeps the cost low.

pub use alloc::collections::*;

use core::fmt;
use core::hash::{BuildHasher, BuildHasherDefault, Hash, Hasher};
use core::iter::FromIterator;
use core::ops::{Deref, DerefMut};

/// 64-bit FNV-1a hasher.
#[derive(Clone, Copy)]
pub struct FnvHasher(u64);

/// FNV-1a offset basis.
const FNV_OFFSET_BASIS: u64 = 0xcbf2_9ce4_8422_2325;
/// FNV-1a prime.
const FNV_PRIME: u64 = 0x0000_0100_0000_01b3;

impl Default for FnvHasher {
    fn default() -> Self {
        Self(FNV_OFFSET_BASIS)
    }
}

impl Hasher for FnvHasher {
    fn finish(&self) -> u64 {
        self.0
    }

    fn write(&mut self, bytes: &[u8]) {
        let mut hash = self.0;
        for byte in bytes {
            hash ^= *byte as u64;
            hash = hash.wrapping_mul(FNV_PRIME);
        }
        self.0 = hash;
    }
}

/// The [`BuildHasher`] used by [`HashMap`].
pub type FnvBuildHasher = BuildHasherDefault<FnvHasher>;

/// A hash map, drop-in compatible with `std::collections::HashMap`.
///
/// All functionality is delegated to [`hashbrown::HashMap`] through `Deref`;
/// the wrapper only fixes the hasher so that `HashMap::new()` needs no type
/// annotation.
pub struct HashMap<K, V, S = FnvBuildHasher> {
    inner: hashbrown::HashMap<K, V, S>,
}

impl<K, V> HashMap<K, V, FnvBuildHasher> {
    /// Creates an empty map.
    pub fn new() -> Self {
        Self {
            inner: hashbrown::HashMap::default(),
        }
    }

    /// Creates an empty map with at least the specified capacity.
    pub fn with_capacity(capacity: usize) -> Self {
        Self {
            inner: hashbrown::HashMap::with_capacity_and_hasher(
                capacity,
                FnvBuildHasher::default(),
            ),
        }
    }
}

impl<K, V, S> Default for HashMap<K, V, S>
where
    S: Default,
{
    fn default() -> Self {
        Self {
            inner: hashbrown::HashMap::default(),
        }
    }
}

impl<K, V, S> Deref for HashMap<K, V, S> {
    type Target = hashbrown::HashMap<K, V, S>;

    fn deref(&self) -> &Self::Target {
        &self.inner
    }
}

impl<K, V, S> DerefMut for HashMap<K, V, S> {
    fn deref_mut(&mut self) -> &mut Self::Target {
        &mut self.inner
    }
}

impl<K, V, S> FromIterator<(K, V)> for HashMap<K, V, S>
where
    K: Eq + Hash,
    S: BuildHasher + Default,
{
    fn from_iter<T: IntoIterator<Item = (K, V)>>(iter: T) -> Self {
        Self {
            inner: hashbrown::HashMap::from_iter(iter),
        }
    }
}

impl<K, V, S> IntoIterator for HashMap<K, V, S> {
    type Item = (K, V);
    type IntoIter = hashbrown::hash_map::IntoIter<K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.into_iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a HashMap<K, V, S> {
    type Item = (&'a K, &'a V);
    type IntoIter = hashbrown::hash_map::Iter<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter()
    }
}

impl<'a, K, V, S> IntoIterator for &'a mut HashMap<K, V, S> {
    type Item = (&'a K, &'a mut V);
    type IntoIter = hashbrown::hash_map::IterMut<'a, K, V>;

    fn into_iter(self) -> Self::IntoIter {
        self.inner.iter_mut()
    }
}

impl<K: fmt::Debug, V: fmt::Debug, S> fmt::Debug for HashMap<K, V, S> {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        fmt::Debug::fmt(&self.inner, f)
    }
}

impl<K: Clone, V: Clone, S: Clone> Clone for HashMap<K, V, S> {
    fn clone(&self) -> Self {
        Self {
            inner: self.inner.clone(),
        }
    }
}
