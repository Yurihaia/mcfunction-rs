use crate::{parser::Language, syntax::TokenKind};

use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedToken<T: TokenKind>(Vec<T>);

impl<T: TokenKind> ExpectedToken<T> {
    pub(crate) fn new(expected: Vec<T>) -> Self {
        ExpectedToken(expected)
    }

    pub fn expected(&self) -> &[T] {
        &self.0
    }
}

impl<T: TokenKind> fmt::Display for ExpectedToken<T> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Expected one of ")?;
        for (i, x) in self.expected().iter().enumerate() {
            if i == 0 {
                write!(f, "'{}'", x)?;
            } else {
                write!(f, ", '{}'", x)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedLit<L: Language>(Cow<'static, [(&'static str, L::GroupType)]>);

impl<L: Language> ExpectedLit<L> {
    pub(crate) fn new(expected: Vec<(&'static str, L::GroupType)>) -> Self {
        ExpectedLit(Cow::Owned(expected))
    }

    pub(crate) fn from_slice(expected: &'static [(&'static str, L::GroupType)]) -> Self {
        ExpectedLit(Cow::Borrowed(expected))
    }

    pub fn expected(&self) -> &[(&str, L::GroupType)] {
        &self.0
    }
}

impl<L: Language> fmt::Display for ExpectedLit<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        write!(f, "Expected one of ")?;
        for (i, (x, _)) in self.expected().iter().enumerate() {
            if i == 0 {
                write!(f, "'{}'", x)?;
            } else {
                write!(f, ", '{}'", x)?;
            }
        }
        Ok(())
    }
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum ParseError<L: Language> {
    Token(ExpectedToken<L::TokenKind>),
    Lit(ExpectedLit<L>),
    Group(L::GroupType),
}

impl<L: Language> fmt::Display for ParseError<L> {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use ParseError::*;

        match self {
            Token(t) => write!(f, "{}", t),
            Lit(l) => write!(f, "{}", l),
            // TODO: Add better error message for group error
            Group(g) => write!(f, "Error while parsing '{:?}'", g),
        }
    }
}

impl<L: Language> From<ExpectedLit<L>> for ParseError<L> {
    fn from(it: ExpectedLit<L>) -> ParseError<L> {
        ParseError::Lit(it)
    }
}

impl<L: Language> From<ExpectedToken<L::TokenKind>> for ParseError<L> {
    fn from(it: ExpectedToken<L::TokenKind>) -> ParseError<L> {
        ParseError::Token(it)
    }
}
