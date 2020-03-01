use crate::{
    error::{ExpectedLit, ExpectedToken},
    syntax::TokenKind,
    tokenset, Ast, LineCol, ParseError, Token, TokenSet,
};

use util::DropBomb;

#[derive(Debug)]
pub struct Parser<'t, 's, L: Language> {
    pub(crate) tokens: &'t [Token<L::TokenKind>],
    events: Vec<Event<L>>,
    src: &'s str,
    skip_ws: bool,
    root: L::GroupType,
}

pub trait Language: 'static + Copy + Clone + PartialEq + Eq + std::hash::Hash {
    type TokenKind: TokenKind;
    type GroupType: std::fmt::Debug + Copy + Eq;

    const ERROR_GROUP: Self::GroupType;
}

impl<'t, 's, L: Language> Parser<'t, 's, L> {
    pub fn new(
        tokens: &'t [Token<L::TokenKind>],
        src: &'s str,
        root: L::GroupType,
        skip_ws: bool,
    ) -> Self {
        Parser {
            tokens,
            events: Vec::new(),
            src,
            skip_ws,
            root,
        }
    }

    pub fn start(&mut self, kind: L::GroupType, skip: StartInfo) -> Marker<'t, L> {
        if self.skip_ws {
            self.skip_ws = false;
            while self.eat_tokens(L::TokenKind::WHITESPACE) {}
            self.skip_ws = true;
        }
        self.start_no_skip(kind, skip)
    }

    pub fn start_no_skip(&mut self, kind: L::GroupType, skip: StartInfo) -> Marker<'t, L> {
        let mk = Marker(
            self.tokens,
            self.events.len(),
            self.skip_ws,
            DropBomb::new("Markers should either be finished or cancelled"),
        );
        self.skip_ws = skip == StartInfo::Skip;
        self.push_event(Event::Start {
            kind,
            join: skip == StartInfo::Join,
        });
        mk
    }

    pub fn finish(&mut self, mut marker: Marker<'t, L>) {
        self.push_event(Event::End {
            linecol: self.tokens[0].span().start(),
            off: self.tokens[0].start(),
        });
        self.skip_ws = marker.2;
        marker.3.defuse();
    }

    pub fn cancel(&mut self, mut marker: Marker<'t, L>) {
        self.events.truncate(marker.1);
        self.tokens = marker.0;
        self.skip_ws = marker.2;
        marker.3.defuse();
    }

    pub fn retype(&mut self, marker: &Marker<'t, L>, kind: L::GroupType, join: bool) {
        match &mut self.events[marker.1] {
            Event::Start { join: j, kind: k } => {
                *k = kind;
                *j = join;
            }
            _ => panic!("Marker is pointing at non-start location"),
        }
    }

    pub fn nth(&self, off: usize) -> L::TokenKind {
        // If accessing past the end of the file, just return the EOF
        self.nth_tk(off).kind()
    }

    pub(crate) fn nth_tk(&self, n: usize) -> Token<L::TokenKind> {
        assert!(n <= 1);
        if self.skip_ws {
            let mut off = 0;
            for _ in 0..=n {
                while tokenset!(self.tokens[off].kind() => L::TokenKind::WHITESPACE) {
                    off += 1;
                }
                off += 1;
            }
            self.tokens[off - 1]
        } else {
            self.tokens[n]
        }
    }

    pub fn at(&self, kind: L::TokenKind) -> bool {
        self.nth(0) == kind
    }

    pub fn not_at(&self, kind: L::TokenKind) -> bool {
        !(self.at(L::TokenKind::EOF) || self.at(kind))
    }

    pub fn bump(&mut self) {
        if self.skip_ws {
            self.skip_ws = false;
            while self.eat_tokens(L::TokenKind::WHITESPACE) {}
            self.skip_ws = true;
        }
        // Never progress past the EOF so there will always be one token in the slice
        // Don't progress over line breaks either so the parser wont crash and burn
        if !self.at(L::TokenKind::EOF) {
            self.push_event(Event::Token(self.nth_tk(0)));
            self.skip();
        }
    }

    pub fn skip(&mut self) {
        if !self.at(L::TokenKind::EOF) {
            self.tokens = &self.tokens[1..];
        }
    }

    pub fn eat(&mut self, kind: L::TokenKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub fn eat_tokens(&mut self, set: TokenSet<L::TokenKind>) -> bool {
        if self.at_tokens(set) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub fn expect(&mut self, kind: L::TokenKind) -> bool {
        if !self.eat(kind) {
            self.events
                .push(Event::Error(ExpectedToken::new(vec![kind]).into()));
            false
        } else {
            true
        }
    }

    // Needed for context aware keywords
    pub fn expect_keyword(&mut self, ex: &'static [(&'static str, L::GroupType)]) -> bool {
        if self.at(L::TokenKind::WORD) {
            let tk = self.nth_tk(0).string(self.src);
            for (x, g) in ex {
                if tk == *x {
                    let mk = self.start(*g, StartInfo::Join);
                    self.bump();
                    self.finish(mk);
                    return true;
                }
            }
            self.events
                .push(Event::Error(ExpectedLit::from_slice(ex).into()));
            false
        } else {
            self.expect(L::TokenKind::WORD);
            false
        }
    }

    pub fn try_token<F: FnOnce(&mut TokenParser<L>) -> Option<()>>(
        &mut self,
        f: F,
        kind: L::GroupType,
    ) -> bool {
        let mk = self.start(kind, StartInfo::Join);
        if f(&mut TokenParser(self)).is_some() {
            self.finish(mk);
            true
        } else {
            self.cancel(mk);
            false
        }
    }

    pub fn at_token<F: FnOnce(&mut TokenParser<L>) -> Option<()>>(&self, f: F) -> bool {
        let mut parser = Parser {
            tokens: self.tokens,
            events: Vec::new(),
            src: self.src,
            skip_ws: self.skip_ws,
            root: self.root,
        };
        let mk = parser.start(L::ERROR_GROUP, StartInfo::Join);
        if f(&mut TokenParser(&mut parser)).is_some() {
            parser.cancel(mk);
            true
        } else {
            parser.cancel(mk);
            false
        }
    }

    pub fn at_keyword(&self, ex: &[(&str, L::GroupType)]) -> bool {
        if self.at(L::TokenKind::WORD) {
            let tk = self.nth_tk(0).string(self.src);
            for (x, _) in ex {
                if tk == *x {
                    return true;
                }
            }
            false
        } else {
            false
        }
    }

    pub fn eat_keyword(&mut self, ex: &'static [(&'static str, L::GroupType)]) -> bool {
        if self.at(L::TokenKind::WORD) {
            let tk = self.nth_tk(0).string(self.src);
            for (x, g) in ex {
                if tk == *x {
                    let mk = self.start(*g, StartInfo::Join);
                    self.bump();
                    self.finish(mk);
                    return true;
                }
            }
            false
        } else {
            false
        }
    }

    pub fn at_tokens(&self, set: TokenSet<L::TokenKind>) -> bool {
        tokenset!(self.nth_tk(0).kind() => set)
    }

    pub fn err_recover(&mut self, group: L::GroupType, set: TokenSet<L::TokenKind>) {
        self.error(group);
        self.bump_recover(set);
    }

    pub fn bump_recover(&mut self, set: TokenSet<L::TokenKind>) -> bool {
        if !self.at_tokens(L::TokenKind::DELIMITERS.union(set)) {
            self.bump();
            false
        } else {
            true
        }
    }

    pub fn lookahead(&mut self) -> Lookahead<'_, 't, 's, L> {
        Lookahead {
            parser: self,
            tried: Vec::new(),
            kw: Vec::new(),
            groups: Vec::new(),
        }
    }

    pub fn error(&mut self, err: L::GroupType) {
        self.push_event(Event::Error(ParseError::Group(err)))
    }

    pub fn add_errors(&mut self, errs: Vec<ParseError<L>>) {
        for err in errs {
            self.push_event(Event::Error(err));
        }
    }

    pub fn change_tokens(&mut self, tks: &'t [Token<L::TokenKind>]) {
        assert!(
            self.tokens[0].end() == tks[0].start(),
            "Token streams must be consecutive: {}, {}",
            self.tokens[0].end(),
            tks[0].start()
        );
        self.tokens = tks;
    }

    fn push_event(&mut self, evt: Event<L>) {
        self.events.push(evt);
    }

    pub fn build(self, save_errors: bool) -> Ast<&'s str, L> {
        crate::ast::build_ast(self.events, self.src, save_errors, self.root)
    }
}

pub struct TokenParser<'p, 't, 's, L: Language>(&'p mut Parser<'t, 's, L>);

impl<'p, 't, 's, L: Language> TokenParser<'p, 't, 's, L> {
    pub fn eat(&mut self, kind: L::TokenKind) -> bool {
        self.expect(kind).is_some()
    }

    pub fn eat_tokens(&mut self, set: TokenSet<L::TokenKind>) -> bool {
        self.expect_tokens(set).is_some()
    }

    pub fn eat_kw(&mut self, kws: &'static [(&'static str, L::GroupType)]) -> bool {
        self.0.eat_keyword(kws)
    }

    pub fn expect(&mut self, kind: L::TokenKind) -> Option<()> {
        if self.0.at(kind) {
            self.0.bump();
            Some(())
        } else {
            None
        }
    }

    pub fn expect_tokens(&mut self, set: TokenSet<L::TokenKind>) -> Option<()> {
        if self.0.at_tokens(set) {
            self.0.bump();
            Some(())
        } else {
            None
        }
    }

    pub fn expect_kw(&mut self, kws: &'static [(&'static str, L::GroupType)]) -> Option<()> {
        if self.0.at_keyword(kws) {
            self.0.bump();
            Some(())
        } else {
            None
        }
    }

    pub fn nth(&self, n: usize) -> L::TokenKind {
        assert!(n <= 1);
        self.0.nth(n)
    }
}

#[derive(Debug)]
pub struct Marker<'t, L: Language>(&'t [Token<L::TokenKind>], usize, bool, DropBomb<&'t str>);

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum StartInfo {
    Join,
    Skip,
    None,
}

#[derive(Debug)]
pub struct Lookahead<'p, 't, 's, L: Language> {
    parser: &'p mut Parser<'t, 's, L>,
    tried: Vec<L::TokenKind>,
    kw: Vec<(&'static str, L::GroupType)>,
    groups: Vec<L::GroupType>,
}

impl<'p, 't, 's, L: Language> Lookahead<'p, 't, 's, L> {
    pub fn at(&mut self, kind: L::TokenKind) -> bool {
        if self.parser.at(kind) {
            true
        } else {
            self.tried.push(kind);
            false
        }
    }

    pub fn at_tks(&self, kind: TokenSet<L::TokenKind>) -> bool {
        self.parser.at_tokens(kind)
    }

    pub fn at_keyword(&mut self, kw: &'static str, gt: L::GroupType) -> bool {
        if self.parser.at_keyword(&[(kw, gt)]) {
            true
        } else {
            self.kw.push((kw, gt));
            false
        }
    }

    pub fn at_keywords(&mut self, ex: &[(&'static str, L::GroupType)]) -> bool {
        if self.parser.at_keyword(ex) {
            true
        } else {
            self.kw.extend_from_slice(ex);
            false
        }
    }

    pub fn group_error(&mut self, gt: L::GroupType) {
        self.groups.push(gt);
    }

    pub fn add_errors(self) {
        self.parser
            .events
            .push(Event::Error(ExpectedToken::new(self.tried).into()));
        self.parser
            .events
            .push(Event::Error(ExpectedLit::new(self.kw).into()));
        for x in self.groups {
            self.parser
                .events
                .push(Event::Error(ParseError::Group(x.into())));
        }
    }

    pub fn get_errors(self) -> Vec<ParseError<L>> {
        let mut out = vec![
            ExpectedToken::new(self.tried).into(),
            ExpectedLit::new(self.kw).into(),
        ];
        for x in self.groups {
            out.push(ParseError::Group(x.into()));
        }
        out
    }
}

#[derive(Debug)]
pub enum Event<L: Language> {
    Start { kind: L::GroupType, join: bool },
    End { linecol: LineCol, off: usize },
    Token(Token<L::TokenKind>),
    Error(ParseError<L>),
}
