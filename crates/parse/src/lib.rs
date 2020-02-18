pub mod ast;
pub mod error;
pub mod grammar;

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

use mcf_util::commands::{Command, CommandNodeType, Commands, Index, ParserType, StringType};

pub fn parse<F: FnOnce(&mut parser::Parser)>(i: &str, f: F) -> ast::AstNode {
    let tokens = lexer::tokenize_str(i);
    let mut parser = parser::Parser::new(&tokens, i);
    f(&mut parser);
    parser.build()
}

pub struct CommandParser<'c> {
    commands: &'c Commands,
}

impl<'c> CommandParser<'c> {
    pub fn parse(&self, p: &mut parser::Parser) {
        while !p.at(TokenKind::Eof) {
            if p.at(TokenKind::LineBreak) {
                p.skip_linebreak();
            } else if p.at(TokenKind::Hash) {
                let cmk = p.start(GroupType::Comment, parser::StartInfo::Join);
                p.bump();
                grammar::message(p);
                p.finish(cmk);
                p.skip_linebreak();
            } else {
                let cmk = p.start(GroupType::Command, parser::StartInfo::None);
                self.parse_command(self.commands.root(), self.commands.root_index(), p);
                p.finish(cmk);
                p.skip_linebreak();
            }
        }
    }

    fn parse_command(&self, c: &Command, ind: Index, p: &mut parser::Parser) {
        if p.at(TokenKind::LineBreak) || p.at(TokenKind::Eof) {
            return;
        }
        match c.node_type() {
            CommandNodeType::Argument { parser_type } => {
                let mk = p.start(GroupType::CommandNode(ind), parser::StartInfo::None);
                match parser_type {
                    ParserType::BlockPos => grammar::coord::coord(p),
                    ParserType::BlockPredicate => grammar::block::predicate(p),
                    ParserType::BlockState => grammar::block::state(p),
                    ParserType::Bool => {
                        if !p.expect_keyword(grammar::BOOLEAN) && p.at(TokenKind::Word) {
                            p.bump();
                        }
                    }
                    ParserType::Color => {
                        // TODO: Add keywords
                        p.expect(TokenKind::Word);
                    }
                    ParserType::ColumnPos => grammar::coord::coord2(p),
                    ParserType::Component => grammar::json::value(p),
                    ParserType::Dimension
                    | ParserType::EntitySummon
                    | ParserType::ItemEnchantment
                    | ParserType::MobEffect
                    | ParserType::Particle
                    | ParserType::ResourceLocation
                    | ParserType::ObjectiveCriteria => grammar::resource_location(p),
                    ParserType::Double | ParserType::Float { properties: _ } => grammar::float(p),
                    ParserType::Entity { properties: _ } => grammar::selector::entity(p),
                    ParserType::EntityAnchor => {
                        p.expect(TokenKind::Word);
                    }
                    ParserType::Function => grammar::function(p),
                    ParserType::GameProfile => grammar::selector::game_profile(p),
                    ParserType::Integer { properties: _ } => grammar::integer(p),
                    ParserType::IntRange => grammar::range(p),
                    ParserType::ItemPredicate => grammar::item_predicate(p),
                    ParserType::ItemSlot => grammar::uq_string(p),
                    ParserType::ItemStack => grammar::item_stack(p),
                    ParserType::Message => grammar::message(p),
                    ParserType::NbtCompoundTag => grammar::nbt::compound(p),
                    ParserType::NbtPath => grammar::nbt::path(p),
                    ParserType::NbtTag => grammar::nbt::value(p),
                    ParserType::Objective => grammar::uq_string(p),
                    ParserType::Operation => {
                        p.eat_tokens(grammar::OPERATION);
                    }
                    ParserType::Rotation => grammar::coord::coord2(p),
                    ParserType::ScoreboardSlot => grammar::uq_string(p),
                    ParserType::ScoreHolder { properties: _ } => grammar::selector::score_holder(p),
                    ParserType::String { properties } => match properties.string_type {
                        StringType::Word => grammar::uq_string(p),
                        StringType::Phrase => grammar::string(p),
                        StringType::Greedy => grammar::message(p),
                    },
                    ParserType::Swizzle => {
                        p.expect(TokenKind::Word);
                    }
                    ParserType::Team => grammar::uq_string(p),
                    ParserType::Time => grammar::time(p),
                    ParserType::Vec2 => grammar::coord::coord2(p),
                    ParserType::Vec3 => grammar::coord::coord(p),
                }
                p.finish(mk);
            }
            CommandNodeType::Literal => {
                let mk = p.start(GroupType::CommandNode(ind), parser::StartInfo::None);
                p.bump();
                p.finish(mk);
            }
            CommandNodeType::Root => (),
        }
        if let CommandNodeType::Root = c.node_type() {
        } else {
            if p.at(TokenKind::LineBreak) || p.at(TokenKind::Eof) {
                return;
            }
            if !p.expect(TokenKind::Whitespace) {
                return;
            }
        }
        let mut best: Option<(Certainty, &Command, Index)> = None;
        for (index, child) in c.children(self.commands) {
            match child.node_type() {
                CommandNodeType::Root => {
                    self.parse_command(self.commands.root(), index, p);
                    return;
                }
                CommandNodeType::Literal => {
                    if p.at_keyword(&[(child.name(), GroupType::Error)]) {
                        self.parse_command(child, index, p);
                        return;
                    } else {
                        for (n, tk) in lexer::PUNCT {
                            if child.name() == *n && p.at(*tk) {
                                self.parse_command(child, index, p);
                                return;
                            }
                        }
                    }
                }
                CommandNodeType::Argument { parser_type } => {
                    let cty = parser_lookahead(p, parser_type);
                    if let Some((c, cmd, _)) = best {
                        if cty > c {
                            best = Some((cty, child, index));
                        } else if cmd.node_type() == child.node_type()
                            && child.children_indices().len() > cmd.children_indices().len()
                        {
                            best = Some((cty, child, index));
                        }
                    } else {
                        best = Some((cty, child, index));
                    }
                }
            }
        }
        if let Some((_, child, index)) = best {
            if !p.at(TokenKind::LineBreak) && !p.at(TokenKind::Eof) {
                self.parse_command(child, index, p);
            }
        } else {
            let errmk = p.start(GroupType::Error, parser::StartInfo::None);
            while !p.at(TokenKind::LineBreak) && !p.at(TokenKind::Eof) {
                p.bump();
            }
            p.finish(errmk);
        }
    }
}

fn parser_lookahead(p: &parser::Parser, arg: ParserType) -> Certainty {
    assert!(Certainty::No < Certainty::Maybe);
    match arg {
        ParserType::BlockPos | ParserType::ColumnPos => {
            if p.at_tokens(grammar::coord::COORD_MODIFIER) {
                return Certainty::Yes;
            } else if p.at_token(grammar::integer_tk) {
                return Certainty::Maybe;
            }
        }
        ParserType::BlockPredicate | ParserType::Function | ParserType::ItemPredicate => {
            if p.at(TokenKind::Hash) || p.at_token(grammar::resource_location_tk) {
                return Certainty::Yes;
            }
        }
        ParserType::BlockState
        | ParserType::Dimension
        | ParserType::EntitySummon
        | ParserType::ItemEnchantment
        | ParserType::ItemStack
        | ParserType::MobEffect
        | ParserType::Particle
        | ParserType::ResourceLocation => {
            if p.at_token(grammar::resource_location_tk) {
                return Certainty::Yes;
            }
        }
        ParserType::Bool => {
            if p.at_keyword(grammar::BOOLEAN) {
                return Certainty::Yes;
            }
        }
        ParserType::Color => {
            if p.at(TokenKind::Word) {
                return Certainty::Probably;
            }
        }
        ParserType::Component => {
            if p.at_tokens(tokenset![
                TokenKind::QuotedString,
                TokenKind::LCurly,
                TokenKind::LBracket
            ]) {
                return Certainty::Yes;
            }
        }
        ParserType::Double | ParserType::Float { properties: _ } => {
            if p.at_token(grammar::float_tk) {
                return Certainty::Yes;
            }
        }
        ParserType::Entity { properties: _ } => {
            if p.at(TokenKind::At) || p.at_token(grammar::uuid_tk) {
                return Certainty::Yes;
            } else if p.at_tokens(grammar::ALLOWED_UQ_STRING) {
                if grammar::ALLOWED_UQ_STRING.contains(p.nth(1)) {
                    return Certainty::Probably;
                }
                return Certainty::Maybe;
            }
        }
        ParserType::EntityAnchor => {
            if p.at_keyword(&[("eyes", GroupType::Error), ("feet", GroupType::Error)]) {
                return Certainty::Yes;
            } else if p.at(TokenKind::Word) {
                return Certainty::Maybe;
            }
        }
        ParserType::GameProfile => {
            if p.at(TokenKind::Word) {
                return Certainty::Yes;
            } else if p.at(TokenKind::Digits) {
                return Certainty::Maybe;
            }
        }
        ParserType::Integer { properties: _ } => {
            if p.at_token(grammar::integer_tk) {
                return Certainty::Yes;
            }
        }
        ParserType::IntRange => {
            if p.at(TokenKind::DotDot) || p.at_token(grammar::integer_tk) {
                return Certainty::Yes;
            }
        }
        ParserType::ItemSlot => {
            if p.at(TokenKind::Word) {
                return Certainty::Yes;
            } else if p.at_tokens(tokenset![
                TokenKind::Digits,
                TokenKind::Dot,
                TokenKind::DotDot
            ]) {
                return Certainty::Maybe;
            }
        }
        ParserType::Message => {
            return Certainty::Maybe;
        }
        ParserType::NbtCompoundTag => {
            if p.at(TokenKind::LCurly) {
                return Certainty::Yes;
            }
        }
        ParserType::NbtPath => {
            if p.at_tokens(tokenset![TokenKind::Word, TokenKind::Dot]) {
                return Certainty::Yes;
            } else if p.at_tokens(tokenset![TokenKind::LBracket, TokenKind::LCurly]) {
                return Certainty::Maybe;
            }
        }
        ParserType::NbtTag => {
            return Certainty::Maybe;
        }
        ParserType::Objective => {
            if p.at_token(grammar::uq_string_ne_tk) {
                return Certainty::Probably;
            }
        }
        ParserType::ObjectiveCriteria => {
            if p.at_tokens(tokenset![
                TokenKind::Word,
                TokenKind::Colon,
                TokenKind::Dot,
                TokenKind::DotDot
            ]) {
                return Certainty::Probably;
            }
        }
        ParserType::Operation => {
            if p.at_tokens(grammar::OPERATION) {
                return Certainty::Yes;
            }
        }
        ParserType::ScoreboardSlot => {
            if p.at_tokens(tokenset![
                TokenKind::Dot,
                TokenKind::DotDot,
                TokenKind::Word
            ]) {
                return Certainty::Maybe;
            }
        }
        ParserType::ScoreHolder { properties: _ } => {
            if p.at_tokens(tokenset![TokenKind::At, TokenKind::Word, TokenKind::Hash])
                || p.at_token(grammar::uuid_tk)
            {
                return Certainty::Yes;
            } else if p.at(TokenKind::Digits) {
                return Certainty::Maybe;
            }
        }
        ParserType::String { properties } => match properties.string_type {
            StringType::Word => {
                if p.at_token(grammar::uq_string_ne_tk) {
                    return Certainty::Probably;
                }
            }
            StringType::Phrase => {
                if p.at(TokenKind::QuotedString) {
                    return Certainty::Yes;
                } else if p.at_token(grammar::uq_string_ne_tk) {
                    return Certainty::Probably;
                }
            }
            StringType::Greedy => {
                return Certainty::Maybe;
            }
        },
        ParserType::Swizzle => {
            if p.at(TokenKind::Word) {
                return Certainty::Maybe;
            }
        }
        ParserType::Team => {
            if p.at(TokenKind::Word) {
                return Certainty::Probably;
            }
        }
        ParserType::Time => {
            if p.at(TokenKind::Digits) {
                return Certainty::Probably;
            }
        }
        ParserType::Vec2 | ParserType::Vec3 | ParserType::Rotation => {
            if p.at_tokens(grammar::coord::COORD_MODIFIER) {
                return Certainty::Yes;
            } else if p.at_token(grammar::float_tk) {
                return Certainty::Maybe;
            }
        }
    }
    Certainty::No
}

#[derive(Copy, Clone, PartialEq, Eq, PartialOrd, Ord)]
enum Certainty {
    No,
    Maybe,
    Probably,
    Yes,
}
