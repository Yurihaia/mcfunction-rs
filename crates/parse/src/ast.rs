use crate::{parser::Language, ParseError, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxKind<L: Language> {
    Root,
    Group(L::GroupType),
    Joined(L::GroupType),
    Token(L::TokenKind),
    Error(ParseError<L>),
}

#[derive(Debug)]
pub struct AstNode<'a, L: Language> {
    kind: SyntaxKind<L>,
    string: &'a str,
    span: Span,
    children: Vec<AstNode<'a, L>>,
}

impl<'a, L: Language> AstNode<'a, L> {
    pub fn new(
        kind: SyntaxKind<L>,
        children: Vec<AstNode<'a, L>>,
        string: &'a str,
        span: Span,
    ) -> Self {
        AstNode {
            kind,
            string,
            span,
            children,
        }
    }

    pub fn kind(&self) -> &SyntaxKind<L> {
        &self.kind
    }

    pub fn children(&self) -> &[AstNode<'a, L>] {
        &self.children
    }

    pub fn string(&self) -> &'a str {
        self.string
    }

    pub fn span(&self) -> Span {
        self.span
    }
}
