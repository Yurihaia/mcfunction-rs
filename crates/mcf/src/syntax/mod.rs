pub use group::McGroupType;
use mcfunction_parse::{
    parser::{Language, Parser, StartInfo},
    tokenset, Ast,
};
pub use tokens::McTokenKind;
use util::commands::{Command, CommandNodeType, Commands, Index, ParserType, StringType};

pub mod cst;
pub mod grammar;
pub mod lexer;
pub mod tokens;

mod group;

#[cfg(test)]
mod testing;

#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct McfLang;

impl Language for McfLang {
    type TokenKind = McTokenKind;
    type GroupType = McGroupType;

    const ERROR_GROUP: Self::GroupType = Self::GroupType::Error;
}

pub type McParser<'a, 'b> = Parser<'a, 'b, McfLang>;

pub struct CommandParser<'c> {
    commands: &'c Commands,
}

pub fn parse_single<F: FnOnce(&mut Parser<McfLang>)>(i: &str, f: F) -> Ast<&str, McfLang> {
    let tokens = lexer::tokenize_str(i);
    assert!(!tokens.is_empty(), "Token stream is empty");
    let mut p = Parser::new(&tokens[0], i, McGroupType::File, false);
    f(&mut p);
    p.build(true)
}

impl<'c> CommandParser<'c> {
    pub fn new(commands: &'c Commands) -> Self {
        CommandParser { commands }
    }

    pub fn parse<'a>(&self, i: &'a str) -> Ast<&'a str, McfLang> {
        let tokens = lexer::tokenize_str(i);
        assert!(!tokens.is_empty(), "Token stream is empty");
        let mut p = Parser::new(&tokens[0], i, McGroupType::File, false);
        self.parse_line(&mut p);
        for line in &tokens[1..] {
            p.change_tokens(&line);
            self.parse_line(&mut p);
        }
        p.build(true)
    }

    fn parse_line(&self, p: &mut McParser) {
        if p.at(McTokenKind::Eof) {
            return;
        }
        if p.at(McTokenKind::Hash) {
            let cmk = p.start(McGroupType::Comment, StartInfo::Join);
            p.bump();
            grammar::message(p);
            p.finish(cmk);
        } else {
            let cmk = p.start(McGroupType::Command, StartInfo::None);
            self.parse_command(self.commands.root(), self.commands.root_index(), p);
            p.finish(cmk);
        }
    }

    fn parse_command(&self, c: &Command, ind: Index, p: &mut McParser) {
        if p.at(McTokenKind::Eof) {
            return;
        }
        match c.node_type() {
            CommandNodeType::Argument { parser_type } => {
                let mk = p.start(McGroupType::CommandNode(ind), StartInfo::None);
                match parser_type {
                    ParserType::BlockPos => grammar::coord::coord(p),
                    ParserType::BlockPredicate => grammar::block::predicate(p),
                    ParserType::BlockState => grammar::block::state(p),
                    ParserType::Bool => {
                        if !p.expect_keyword(grammar::BOOLEAN) && p.at(McTokenKind::Word) {
                            p.bump();
                        }
                    }
                    ParserType::Color => {
                        // TODO: Add keywords
                        p.expect(McTokenKind::Word);
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
                        p.expect(McTokenKind::Word);
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
                        p.expect(McTokenKind::Word);
                    }
                    ParserType::Team => grammar::uq_string(p),
                    ParserType::Time => grammar::time(p),
                    ParserType::Vec2 => grammar::coord::coord2(p),
                    ParserType::Vec3 => grammar::coord::coord(p),
                }
                p.finish(mk);
            }
            CommandNodeType::Literal => {
                let mk = p.start(McGroupType::CommandNode(ind), StartInfo::None);
                p.bump();
                p.finish(mk);
            }
            CommandNodeType::Root => (),
        }
        if let CommandNodeType::Root = c.node_type() {
        } else {
            if p.at(McTokenKind::Eof) {
                return;
            }
            if !p.expect(McTokenKind::Whitespace) {
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
                    if p.at_keyword(&[(child.name(), McGroupType::Error)]) {
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
                        if cty > c
                            || (cmd.node_type() == child.node_type()
                                && child.children_indices().len() > cmd.children_indices().len())
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
            if !p.at(McTokenKind::Eof) {
                self.parse_command(child, index, p);
            }
        } else {
            let errmk = p.start(McGroupType::Error, StartInfo::None);
            while !p.at(McTokenKind::Eof) {
                p.bump();
            }
            p.finish(errmk);
        }
    }
}

fn parser_lookahead(p: &McParser, arg: ParserType) -> Certainty {
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
            if p.at(McTokenKind::Hash) || p.at_token(grammar::resource_location_tk) {
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
            if p.at(McTokenKind::Word) {
                return Certainty::Probably;
            }
        }
        ParserType::Component => {
            if p.at_tokens(tokenset![
                McTokenKind::QuotedString,
                McTokenKind::LCurly,
                McTokenKind::LBracket
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
            if p.at(McTokenKind::At) || p.at_token(grammar::uuid_tk) {
                return Certainty::Yes;
            } else if p.at_tokens(grammar::ALLOWED_UQ_STRING) {
                if tokenset!(p.nth(1) => grammar::ALLOWED_UQ_STRING) {
                    return Certainty::Probably;
                }
                return Certainty::Maybe;
            }
        }
        ParserType::EntityAnchor => {
            if p.at_keyword(&[("eyes", McGroupType::Error), ("feet", McGroupType::Error)]) {
                return Certainty::Yes;
            } else if p.at(McTokenKind::Word) {
                return Certainty::Maybe;
            }
        }
        ParserType::GameProfile => {
            if p.at(McTokenKind::Word) {
                return Certainty::Yes;
            } else if p.at(McTokenKind::Digits) {
                return Certainty::Maybe;
            }
        }
        ParserType::Integer { properties: _ } => {
            if p.at_token(grammar::integer_tk) {
                return Certainty::Yes;
            }
        }
        ParserType::IntRange => {
            if p.at(McTokenKind::DotDot) || p.at_token(grammar::integer_tk) {
                return Certainty::Yes;
            }
        }
        ParserType::ItemSlot => {
            if p.at(McTokenKind::Word) {
                return Certainty::Yes;
            } else if p.at_tokens(tokenset![
                McTokenKind::Digits,
                McTokenKind::Dot,
                McTokenKind::DotDot
            ]) {
                return Certainty::Maybe;
            }
        }
        ParserType::Message => {
            return Certainty::Maybe;
        }
        ParserType::NbtCompoundTag => {
            if p.at(McTokenKind::LCurly) {
                return Certainty::Yes;
            }
        }
        ParserType::NbtPath => {
            if p.at_tokens(tokenset![McTokenKind::Word, McTokenKind::Dot]) {
                return Certainty::Yes;
            } else if p.at_tokens(tokenset![McTokenKind::LBracket, McTokenKind::LCurly]) {
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
                McTokenKind::Word,
                McTokenKind::Colon,
                McTokenKind::Dot,
                McTokenKind::DotDot
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
                McTokenKind::Dot,
                McTokenKind::DotDot,
                McTokenKind::Word
            ]) {
                return Certainty::Maybe;
            }
        }
        ParserType::ScoreHolder { properties: _ } => {
            if p.at_tokens(tokenset![
                McTokenKind::At,
                McTokenKind::Word,
                McTokenKind::Hash
            ]) || p.at_token(grammar::uuid_tk)
            {
                return Certainty::Yes;
            } else if p.at(McTokenKind::Digits) {
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
                if p.at(McTokenKind::QuotedString) {
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
            if p.at(McTokenKind::Word) {
                return Certainty::Maybe;
            }
        }
        ParserType::Team => {
            if p.at(McTokenKind::Word) {
                return Certainty::Probably;
            }
        }
        ParserType::Time => {
            if p.at(McTokenKind::Digits) {
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
