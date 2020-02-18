use crate::{ParseError, Span, TokenKind};

use mcf_util::commands::Index;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum GroupType {
    CommandNode(Index),
    Command,
    // NBT groups
    NbtCompound,
    NbtSequence,
    NbtNumber,
    NbtString,
    NbtBoolean,
    // NBT Keywords
    NbtSuffixB,
    NbtSuffixS,
    NbtSuffixL,
    NbtSuffixF,
    NbtSuffixD,
    NbtPrefixB,
    NbtPrefixI,
    NbtPrefixL,
    // NBT Compound
    NbtCompoundEntry,

    NbtPath,
    NbtPathSegment,
    NbtPathIndex,

    // Selector groups
    Selector,
    SelectorArgument,
    SelectorArgumentEntry,
    SelectorArgumentMap,
    SelectorArgumentMapEntry,
    // Selector keywords
    SelectorModP,
    SelectorModA,
    SelectorModR,
    SelectorModS,
    SelectorModE,

    // Block states
    BlockState,
    BlockStateArguments,

    ItemStack,
    ItemPredicate,

    Comment,

    Function,

    JsonObject,
    JsonObjectEntry,
    JsonList,
    JsonNull,

    Integer,
    Float,
    UnquotedString,
    ResourceLocation,
    Range,
    Uuid,
    Time,

    TimeS,
    TimeT,
    TimeD,

    Coord,
    CoordPart,

    // Keywords
    BooleanTrue,
    BooleanFalse,

    FloatSciExpUpper,
    FloatSciExpLower,

    // Special error type that signifies an error
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxKind {
    Root,
    Group(GroupType),
    Joined(GroupType),
    Token(TokenKind),
    Error(ParseError),
}

#[derive(Debug)]
pub struct AstNode<'a> {
    kind: SyntaxKind,
    string: &'a str,
    span: Span,
    children: Vec<AstNode<'a>>,
}

impl<'a> AstNode<'a> {
    pub fn new(kind: SyntaxKind, children: Vec<AstNode<'a>>, string: &'a str, span: Span) -> Self {
        AstNode {
            kind,
            string,
            span,
            children,
        }
    }

    pub fn kind(&self) -> &SyntaxKind {
        &self.kind
    }

    pub fn children(&self) -> &[AstNode<'a>] {
        &self.children
    }

    pub fn string(&self) -> &'a str {
        self.string
    }

    pub fn span(&self) -> Span {
        self.span
    }
}
