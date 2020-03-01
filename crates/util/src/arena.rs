use std::fmt::Debug;
use std::marker::PhantomData;
use std::{hash::Hash, ops};

use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, PartialEq, Eq, Deserialize, Serialize)]
pub struct Arena<I, T>(Vec<T>, /* *mut for invariance */ PhantomData<*mut I>);

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash, Deserialize, Serialize)]
pub struct RawId(usize);

impl<I: ArenaId, T> Arena<I, T> {
    pub fn new() -> Self {
        Self(Vec::new(), PhantomData)
    }

    pub fn push(&mut self, t: T) -> I {
        let ind = self.0.len();
        self.0.push(t);
        I::from(RawId(ind))
    }

    pub fn indices(&self) -> impl Iterator<Item = I> {
        (0..self.0.len()).map(|ind| I::from(RawId(ind)))
    }

    pub fn entries(&self) -> impl Iterator<Item = (&T, I)> {
        self.indices().map(move |v| (&self[v], v))
    }

    pub fn iter(&self) -> impl Iterator<Item = &T> {
        self.0.iter()
    }

    pub fn from_iter(iter: impl Iterator<Item = T>) -> Self {
        Self(iter.collect(), PhantomData)
    }
}

pub trait ArenaId: Into<RawId> + From<RawId> + Copy + Debug + Hash + Eq {}

impl<I: ArenaId, T> ops::Index<I> for Arena<I, T> {
    type Output = T;
    fn index(&self, index: I) -> &Self::Output {
        &self.0[index.into().0]
    }
}

impl<I: ArenaId, T> ops::IndexMut<I> for Arena<I, T> {
    fn index_mut(&mut self, index: I) -> &mut Self::Output {
        &mut self.0[index.into().0]
    }
}

impl<I, T> IntoIterator for Arena<I, T> {
    type Item = T;
    type IntoIter = std::vec::IntoIter<T>;
    fn into_iter(self) -> Self::IntoIter {
        self.0.into_iter()
    }
}

impl From<RawId> for usize {
    fn from(id: RawId) -> Self {
        id.0
    }
}

#[macro_export]
macro_rules! arena_id {
    ($id:ident) => {
        impl ::std::convert::From<$crate::arena::RawId> for $id {
            fn from(it: $crate::arena::RawId) -> Self {
                Self(it)
            }
        }

        impl ::std::convert::From<$id> for $crate::arena::RawId {
            fn from(it: $id) -> Self {
                it.0
            }
        }

        impl ::std::fmt::Debug for $id {
            fn fmt(&self, f: &mut ::std::fmt::Formatter) -> ::std::fmt::Result {
                write!(f, concat!(stringify!($id), "({})"), usize::from(self.0))
            }
        }

        impl $crate::arena::ArenaId for $id {}
    };
}
