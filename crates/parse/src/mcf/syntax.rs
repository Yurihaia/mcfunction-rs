use crate::{syntax::TokenKind, TokenSet};
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum McTokenKind {
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
    Hash,
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

impl TokenKind for McTokenKind {
    const WHITESPACE: Self = Self::Whitespace;
    const EOF: Self = Self::Eof;
    const LINE_BREAK: Self = Self::LineBreak;
    const WORD: Self = Self::Word;
    const DELIMITERS: TokenSet<Self> = TokenSet::empty();
}

impl From<McTokenKind> for u8 {
    fn from(it: McTokenKind) -> Self {
        it as u8
    }
}

impl fmt::Display for McTokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use McTokenKind::*;

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
                Hash => "#",
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
