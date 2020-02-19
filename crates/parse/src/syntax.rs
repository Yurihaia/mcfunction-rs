use super::Span;

use std::marker::PhantomData;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Token<K: TokenKind> {
    span: Span,
    kind: K,
    off: (usize, usize),
}

impl<K: TokenKind> Token<K> {
    pub fn new(kind: K, span: Span, start: usize, end: usize) -> Self {
        Token {
            kind,
            span,
            off: (start, end),
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn kind(&self) -> K {
        self.kind
    }

    pub fn string<'a>(&self, i: &'a str) -> &'a str {
        &i[self.off.0..self.off.1]
    }

    pub fn start(&self) -> usize {
        self.off.0
    }

    pub fn end(&self) -> usize {
        self.off.1
    }
}

pub trait TokenKind: Sized + std::fmt::Debug + std::fmt::Display + Copy + Eq + Into<u8> {
    const WHITESPACE: Self;
    const EOF: Self;
    const WORD: Self;
    const DELIMITERS: TokenSet<Self>;
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct TokenSet<T>(u128, PhantomData<*const T>);

impl<T> TokenSet<T> {
    pub const fn empty() -> Self {
        TokenSet(0, PhantomData)
    }

    pub const fn singleton(kind: u8) -> Self {
        TokenSet(Self::flag(kind), PhantomData)
    }

    pub fn from_iter(iter: impl IntoIterator<Item = impl Into<u8>>) -> Self {
        let mut out = 0u128;
        for x in iter {
            out |= Self::flag(x.into());
        }
        TokenSet(out, PhantomData)
    }

    pub const fn union(self, other: TokenSet<T>) -> Self {
        TokenSet(self.0 | other.0, PhantomData)
    }

    pub const fn contains(self, kind: u8) -> bool {
        (Self::flag(kind) & self.0) != 0
    }

    const fn flag(kind: u8) -> u128 {
        1u128 << kind
    }
}

#[macro_export]
macro_rules! tokenset {
    ($($ex:expr),*) => {
        $crate::TokenSet::empty()$(.union($crate::TokenSet::singleton($ex as u8)))*
    };
    ($ex:expr => $ts:expr) => {{
        let ts: $crate::TokenSet<_> = $ts;
        ts.contains($ex.into())
    }}
}
