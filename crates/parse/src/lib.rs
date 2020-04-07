#![deny(unsafe_code)]
pub mod ast;
pub mod error;
pub mod parser;

mod span;
mod syntax;

pub use ast::{Ast, AstView, SyntaxKind};
pub use error::ParseError;
pub use span::{LineCol, Span};
pub use syntax::{Token, TokenKind, TokenSet};
