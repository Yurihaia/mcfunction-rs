pub mod ast;
pub mod error;
pub mod grammar;

pub(crate) mod dropbomb;

mod lexer;
mod parser;
mod span;
mod syntax;

#[cfg(test)]
mod testing;

pub use ast::{AstNode, GroupType, SyntaxKind};
pub use error::ParseError;
pub use lexer::tokenize_str;
pub use span::{LineCol, Span};
pub use syntax::{Token, TokenKind, TokenSet};

pub fn parse<F: FnOnce(&mut parser::Parser)>(i: &str, f: F) -> ast::AstNode {
    let tokens = lexer::tokenize_str(i);
    let mut parser = parser::Parser::new(&tokens, i);
    f(&mut parser);
    parser.build()
}
