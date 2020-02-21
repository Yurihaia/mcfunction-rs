use mcfunction_parse::{tokenset, TokenKind, TokenSet};
use std::fmt;

#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
#[repr(u8)]
pub enum NdTokenKind {
    // Single Character Punctuation
    Comma = 0,
    At,
    Colon,
    Bar,
    Eq,
    Slash,
    Dot,
    Semicolon,
    // Double Character Punctuation
    DotDot,
    ColonColon,
    // Keyword Tokens
    ByteKw,
    ShortKw,
    IntKw,
    LongKw,
    FloatKw,
    DoubleKw,
    StringKw,
    BooleanKw,
    ModKw,
    CompoundKw,
    EnumKw,
    InjectKw,
    SuperKw,
    ExtendsKw,
    ExportKw,
    UseKw,
    DescribesKw,
    IdKw,
    // Delimiters
    LBracket,
    RBracket,
    LCurly,
    RCurly,
    LParen,
    RParen,
    // Arbitrary Length
    QuotedString, // Does not have to be terminated, and will end at the end of any line
    Ident,        // String of letters or underscores
    Whitespace,
    Float,
    Comment,
    DocComment,
    // Other
    Invalid, // Used to mark a lexing error
    Eof,     // Used to mark the end of a file
}

impl NdTokenKind {
    pub fn is_keyword(&self) -> bool {
        use NdTokenKind::*;
        match self {
            ByteKw | ShortKw | IntKw | LongKw | FloatKw | DoubleKw | StringKw | BooleanKw
            | ModKw | CompoundKw | EnumKw | InjectKw | SuperKw | ExtendsKw | ExportKw | UseKw
            | DescribesKw => true,
            _ => false,
        }
    }
}

impl TokenKind for NdTokenKind {
    const WHITESPACE: TokenSet<Self> = tokenset![Self::Whitespace, Self::Comment, Self::DocComment];
    const EOF: Self = Self::Eof;
    const WORD: Self = Self::Ident;
    const DELIMITERS: TokenSet<Self> = TokenSet::empty();
}

impl From<NdTokenKind> for u8 {
    fn from(it: NdTokenKind) -> Self {
        it as u8
    }
}

impl fmt::Display for NdTokenKind {
    fn fmt(&self, f: &mut fmt::Formatter) -> fmt::Result {
        use NdTokenKind::*;

        write!(
            f,
            "{}",
            match self {
                Comma => ",",
                Colon => ":",
                At => "@",
                Bar => "|",
                Eq => "=",
                Slash => "/",
                Dot => ".",
                Semicolon => ";",
                DotDot => "..",
                ColonColon => "::",
                LBracket => "[",
                RBracket => "]",
                LCurly => "{",
                RCurly => "}",
                LParen => "(",
                RParen => ")",

                ByteKw => "byte",
                ShortKw => "short",
                IntKw => "int",
                LongKw => "long",
                FloatKw => "float",
                DoubleKw => "double",
                StringKw => "string",
                BooleanKw => "boolean",
                ModKw => "mod",
                CompoundKw => "compound",
                EnumKw => "enum",
                InjectKw => "inject",
                SuperKw => "super",
                ExtendsKw => "extends",
                ExportKw => "export",
                UseKw => "use",
                DescribesKw => "describes",
                IdKw => "id",

                QuotedString => "Quoted String",
                Ident => "Ident",
                Whitespace => "Whitespace",
                Float => "Float",
                Invalid => "Invalid",
                Comment => "Comment",
                DocComment => "DocComment",
                Eof => "EOF",
            }
        )
    }
}
