use crate::{GroupType, TokenKind};

use std::borrow::Cow;
use std::fmt;

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ExpectedToken(Vec<TokenKind>);

impl ExpectedToken {
    pub(crate) fn new(expected: Vec<TokenKind>) -> Self {
        ExpectedToken(expected)
    }

    pub fn expected(&self) -> &[TokenKind] {
        &self.0
    }
}

impl fmt::Display for ExpectedToken {
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
pub struct ExpectedLit(Cow<'static, [(&'static str, GroupType)]>);

impl ExpectedLit {
    pub(crate) fn new(expected: Vec<(&'static str, GroupType)>) -> Self {
        ExpectedLit(Cow::Owned(expected))
    }

    pub(crate) fn from_slice(expected: &'static [(&'static str, GroupType)]) -> Self {
        ExpectedLit(Cow::Borrowed(expected))
    }

    pub fn expected(&self) -> &[(&str, GroupType)] {
        &self.0
    }
}

impl fmt::Display for ExpectedLit {
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
pub enum ParseError {
    Token(ExpectedToken),
    Lit(ExpectedLit),
    Group(GroupType),
}

impl fmt::Display for ParseError {
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

impl From<ExpectedLit> for ParseError {
    fn from(it: ExpectedLit) -> ParseError {
        ParseError::Lit(it)
    }
}

impl From<ExpectedToken> for ParseError {
    fn from(it: ExpectedToken) -> ParseError {
        ParseError::Token(it)
    }
}

impl From<GroupType> for ParseError {
    fn from(it: GroupType) -> ParseError {
        ParseError::Group(it)
    }
}
