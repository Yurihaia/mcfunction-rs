use super::Span;

use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub struct Token {
    span: Span,
    kind: TokenKind,
    off: (usize, usize),
}

impl Token {
    pub fn new(kind: TokenKind, span: Span, start: usize, end: usize) -> Self {
        Token {
            kind,
            span,
            off: (start, end),
        }
    }

    pub fn span(&self) -> Span {
        self.span
    }

    pub fn kind(&self) -> TokenKind {
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

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum TokenKind {
    // Single Character Punctuation
    Comma = 0,
    Dot,
    Colon,
    Semicolon,
    At,
    Excl,
    Eq,
    Lt,
    Gt,
    Slash,
    Tilde,
    Caret,
    Plus,
    Dash,
    // Double Character Punctuation
    DotDot,
    Lte,
    Gte,
    Swap,
    AddAssign,
    SubAssign,
    MulAssign,
    DivAssign,
    ModAssign,
    // Delimiters
    LBracket,
    RBracket,
    LCurly,
    RCurly,
    // Arbitrary Length
    QuotedString, // Does not have to be terminated, and will end at the end of any line
    Word,         // String of letters or underscores
    Whitespace,   // Non linebreak whitespace
    LineBreak,    // LF, CRLF, CR
    Digits,
    // Other
    Invalid, // Used to mark a lexing error
    Eof,     // Used to mark the end of a file
}

impl fmt::Display for TokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use TokenKind::*;

        write!(
            f,
            "{}",
            match self {
                Comma => ",",
                Dot => ".",
                Colon => ":",
                Semicolon => ";",
                At => "@",
                Excl => "!",
                Eq => "=",
                Lt => "<",
                Gt => ">",
                Slash => "/",
                Tilde => "~",
                Caret => "^",
                Plus => "+",
                Dash => "-",
                DotDot => "..",
                Lte => "<=",
                Gte => ">=",
                Swap => "><",
                AddAssign => "+=",
                SubAssign => "-=",
                MulAssign => "*=",
                DivAssign => "/=",
                ModAssign => "%=",
                LBracket => "[",
                RBracket => "]",
                LCurly => "{",
                RCurly => "}",
                QuotedString => "Quoted String",
                Word => "Word",
                Whitespace => "Whitespace",
                LineBreak => "Linebreak",
                Digits => "Digits",
                Invalid => "Invalid",
                Eof => "EOF",
            }
        )
    }
}

#[derive(Copy, Clone, PartialEq, Eq, Hash)]
pub struct TokenSet(u128);

impl TokenSet {
    pub const fn empty() -> Self {
        TokenSet(0)
    }

    pub const fn singleton(kind: TokenKind) -> Self {
        TokenSet(Self::flag(kind))
    }

    pub fn from_iter(iter: impl IntoIterator<Item = TokenKind>) -> Self {
        let mut out = 0u128;
        for x in iter {
            out |= Self::flag(x);
        }
        TokenSet(out)
    }

    pub const fn union(self, other: TokenSet) -> Self {
        TokenSet(self.0 | other.0)
    }

    pub const fn contains(self, kind: TokenKind) -> bool {
        (Self::flag(kind) & self.0) != 0
    }

    const fn flag(kind: TokenKind) -> u128 {
        1u128 << kind as u8
    }
}

#[macro_export]
macro_rules! tokenset {
    ($($ex:expr),*) => {
        TokenSet::empty()$(.union(TokenSet::singleton($ex)))*
    };
}
