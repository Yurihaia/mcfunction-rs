use mcfunction_parse::parser::{Language, Parser};

pub mod grammar;
pub mod group;
pub mod lexer;
pub mod tokens;

#[cfg(test)]
mod testing;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub struct NbtdocLang;

impl Language for NbtdocLang {
    type TokenKind = tokens::NdTokenKind;
    type GroupType = group::NdGroupType;
    const ERROR_GROUP: Self::GroupType = group::NdGroupType::Error;
}

pub type NdParser<'a, 'b> = Parser<'a, 'b, NbtdocLang>;
