use crate::{
    ast::{AstNode, GroupType, SyntaxKind},
    dropbomb::DropBomb,
    error::{ExpectedLit, ExpectedToken},
    LineCol, ParseError, Span, Token, TokenKind, TokenSet,
};

#[derive(Debug)]
pub struct Parser<'t, 's> {
    tokens: &'t [Token],
    events: Vec<Event>,
    src: &'s str,
    skip_ws: bool,
}

const DELIMITERS: TokenSet = TokenSet::singleton(TokenKind::LBracket)
    .union(TokenSet::singleton(TokenKind::RBracket))
    .union(TokenSet::singleton(TokenKind::LCurly))
    .union(TokenSet::singleton(TokenKind::RCurly));

impl<'t, 's> Parser<'t, 's> {
    // All tokens need to be from the same string slice, hence the unsafety
    pub(crate) fn new(tokens: &'t [Token], src: &'s str) -> Self {
        Parser {
            tokens,
            events: Vec::new(),
            src,
            skip_ws: false,
        }
    }

    pub fn start(&mut self, kind: GroupType, join: bool, skip: bool) -> Marker<'t> {
        if self.skip_ws {
            self.skip_ws();
        }
        let mk = Marker(
            self.tokens,
            self.events.len(),
            self.skip_ws,
            DropBomb::new("Markers should either be finished or cancelled"),
        );
        self.skip_ws = skip;
        self.events.push(Event::Start { kind, join });
        mk
    }

    pub fn finish(&mut self, mut marker: Marker<'t>) {
        self.events.push(Event::End {
            linecol: self.tokens[0].span().start(),
            off: self.tokens[0].start(),
        });
        self.skip_ws = marker.2;
        marker.3.defuse();
    }

    pub fn cancel(&mut self, mut marker: Marker<'t>) {
        self.events.truncate(marker.1);
        self.tokens = marker.0;
        self.skip_ws = marker.2;
        marker.3.defuse();
    }

    pub fn retype(&mut self, marker: &Marker<'t>, kind: GroupType, join: bool) {
        match &mut self.events[marker.1] {
            Event::Start { join: j, kind: k } => {
                *k = kind;
                *j = join;
            }
            _ => panic!("Marker is pointing at non-start location"),
        }
    }

    pub fn nth(&self, off: usize) -> TokenKind {
        // If accessing past the end of the file, just return the EOF
        self.nth_tk(off).kind()
    }

    fn nth_tk(&self, n: usize) -> Token {
        assert!(n <= 1);
        match n {
            0 => next_tk(self.tokens, self.skip_ws)[0],
            1 => next_tk(&next_tk(self.tokens, self.skip_ws)[1..], self.skip_ws)[0],
            _ => unreachable!(),
        }
    }

    pub fn at(&self, kind: TokenKind) -> bool {
        self.nth(0) == kind
    }

    pub fn bump(&mut self) {
        let tk = self.nth_tk(0);
        // Never progress past the EOF so there will always be one token in the slice
        if !self.at(TokenKind::Eof) {
            self.events.push(Event::Token(tk));
            self.skip();
        }
    }

    pub fn skip(&mut self) {
        if !self.at(TokenKind::Eof) {
            self.tokens = &next_tk(self.tokens, self.skip_ws)[1..];
        }
    }

    pub fn eat(&mut self, kind: TokenKind) -> bool {
        if self.at(kind) {
            self.bump();
            true
        } else {
            false
        }
    }

    pub fn expect(&mut self, kind: TokenKind) -> bool {
        if !self.eat(kind) {
            self.events
                .push(Event::Error(ExpectedToken::new(vec![kind]).into()));
            false
        } else {
            true
        }
    }

    // Needed for context aware keywords
    pub fn expect_keyword(&mut self, ex: &'static [(&'static str, GroupType)]) -> bool {
        if self.at(TokenKind::Word) {
            let tk = self.nth_tk(0).string(self.src);
            for (x, g) in ex {
                if tk == *x {
                    let mk = self.start(*g, true, false);
                    self.bump();
                    self.finish(mk);
                    return true;
                }
            }
            self.events
                .push(Event::Error(ExpectedLit::from_slice(ex).into()));
            false
        } else {
            self.expect(TokenKind::Word);
            false
        }
    }

    pub fn at_keyword(&self, ex: &[(&str, GroupType)]) -> bool {
        if self.at(TokenKind::Word) {
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

    pub fn eat_keyword(&mut self, ex: &'static [(&'static str, GroupType)]) -> bool {
        if self.at(TokenKind::Word) {
            let tk = self.nth_tk(0).string(self.src);
            for (x, g) in ex {
                if tk == *x {
                    let mk = self.start(*g, true, false);
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

    pub fn at_tokens(&self, set: TokenSet) -> bool {
        set.contains(self.nth_tk(0).kind())
    }

    pub fn err_recover(&mut self, group: GroupType, set: TokenSet) {
        self.error(group);
        self.bump_recover(set);
    }

    pub fn bump_recover(&mut self, set: TokenSet) -> bool {
        if !self.at_tokens(DELIMITERS.union(set)) {
            self.bump();
            false
        } else {
            true
        }
    }

    pub fn lookahead(&mut self) -> Lookahead<'_, 't, 's> {
        Lookahead {
            parser: self,
            tried: Vec::new(),
            kw: Vec::new(),
            groups: Vec::new(),
        }
    }

    pub fn skip_ws(&mut self) {
        if self.tokens[0].kind() == TokenKind::Whitespace {
            self.tokens = &self.tokens[1..];
        }
    }

    pub fn error(&mut self, err: GroupType) {
        self.events.push(Event::Error(err.into()))
    }

    pub fn build(self) -> AstNode<'s> {
        #[derive(Debug)]
        struct ParentNode {
            kind: Option<GroupType>,
            children: Vec<PartialNode>,
            join: bool,
            spans: Option<(Span, usize, usize)>,
        }

        #[derive(Debug)]
        enum PartialNode {
            Error(ParseError),
            Group(ParentNode),
            Token(Token),
        }

        let mut stack: Vec<ParentNode> = vec![ParentNode {
            kind: None,
            children: Vec::new(),
            join: false,
            spans: None,
        }];

        for event in self.events {
            match event {
                Event::Start { kind, join } => {
                    stack.push(ParentNode {
                        kind: Some(kind),
                        children: Vec::new(),
                        join,
                        spans: None,
                    });
                }
                Event::End { linecol, off } => {
                    let mut node = stack.pop().unwrap();
                    if node.spans.is_none() {
                        node.spans = Some((Span::new(linecol, linecol), off, off));
                    }
                    let parent = stack.last_mut().unwrap();
                    let (span, start, end) = node.spans.unwrap();
                    match &mut parent.spans {
                        Some((sp, s, e)) => {
                            *sp = sp.union(&span);
                            *s = (*s).min(start);
                            *e = (*e).max(end);
                        }
                        v @ None => *v = Some((span, start, end)),
                    }
                    parent.children.push(PartialNode::Group(node))
                }
                Event::Error(err) => stack
                    .last_mut()
                    .unwrap()
                    .children
                    .push(PartialNode::Error(err)),
                Event::Token(tk) => {
                    let parent = stack.last_mut().unwrap();
                    match &mut parent.spans {
                        Some((sp, s, e)) => {
                            *sp = sp.union(&tk.span());
                            *s = (*s).min(tk.start());
                            *e = (*e).max(tk.end());
                        }
                        v @ None => *v = Some((tk.span(), tk.start(), tk.end())),
                    }
                    parent.children.push(PartialNode::Token(tk));
                }
            }
        }

        fn convert<'a>(node: ParentNode, src: &'a str) -> AstNode<'a> {
            let (span, start, end) = node.spans.unwrap();
            let mut children = Vec::with_capacity(node.children.len());
            let mut iter = node.children.into_iter().peekable();
            while let Some(child) = iter.next() {
                match child {
                    PartialNode::Token(tk) => {
                        children.push(AstNode::new(
                            SyntaxKind::Token(tk.kind()),
                            Vec::new(),
                            tk.string(src),
                            tk.span(),
                        ));
                    }
                    PartialNode::Group(node) => children.push(convert(node, src)),
                    PartialNode::Error(err) => {
                        let mut errs = vec![err];
                        while let Some(v) = iter.peek() {
                            match v {
                                PartialNode::Error(_) => match iter.next() {
                                    Some(PartialNode::Error(err)) => errs.push(err),
                                    _ => unreachable!(),
                                },
                                _ => break,
                            };
                        }
                        match iter.peek() {
                            None => {
                                for err in errs {
                                    children.push(AstNode::new(
                                        SyntaxKind::Error(err),
                                        Vec::new(),
                                        &src[end..end],
                                        Span::new(span.end(), span.end()),
                                    ))
                                }
                            }
                            Some(PartialNode::Token(tk)) => {
                                for err in errs {
                                    children.push(AstNode::new(
                                        SyntaxKind::Error(err),
                                        Vec::new(),
                                        tk.string(src),
                                        tk.span(),
                                    ))
                                }
                            }
                            Some(PartialNode::Group(_)) => {
                                if let Some(PartialNode::Group(parent)) = iter.next() {
                                    let astnode = convert(parent, src);
                                    for err in errs {
                                        children.push(AstNode::new(
                                            SyntaxKind::Error(err),
                                            Vec::new(),
                                            astnode.string(),
                                            astnode.span(),
                                        ));
                                    }
                                    children.push(astnode);
                                } else {
                                    unreachable!()
                                }
                            }
                            _ => unreachable!(),
                        }
                    }
                }
            }
            AstNode::new(
                node.kind
                    .map(if node.join {
                        SyntaxKind::Joined
                    } else {
                        SyntaxKind::Group
                    })
                    .unwrap_or(SyntaxKind::Root),
                children,
                &src[start..end],
                span,
            )
        }

        assert!(stack.len() == 1);
        convert(stack.pop().unwrap(), self.src)
    }
}

#[derive(Debug)]
pub struct Marker<'t>(&'t [Token], usize, bool, DropBomb<&'t str>);

#[derive(Debug)]
pub struct Lookahead<'p, 't, 's> {
    parser: &'p mut Parser<'t, 's>,
    tried: Vec<TokenKind>,
    kw: Vec<(&'static str, GroupType)>,
    groups: Vec<GroupType>,
}

impl<'p, 't, 's> Lookahead<'p, 't, 's> {
    pub fn at(&mut self, kind: TokenKind) -> bool {
        if self.parser.at(kind) {
            true
        } else {
            self.tried.push(kind);
            false
        }
    }

    pub fn at_tks(&self, kind: TokenSet) -> bool {
        self.parser.at_tokens(kind)
    }

    pub fn at_keyword(&mut self, kw: &'static str, gt: GroupType) -> bool {
        if self.parser.at_keyword(&[(kw, gt)]) {
            true
        } else {
            self.kw.push((kw, gt));
            false
        }
    }

    pub fn at_keywords(&mut self, ex: &[(&'static str, GroupType)]) -> bool {
        if self.parser.at_keyword(ex) {
            true
        } else {
            self.kw.extend_from_slice(ex);
            false
        }
    }

    pub fn group_error(&mut self, gt: GroupType) {
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
            self.parser.events.push(Event::Error(x.into()));
        }
    }

    pub fn get_errors(self) -> Vec<ParseError> {
        let mut out = vec![
            ExpectedToken::new(self.tried).into(),
            ExpectedLit::new(self.kw).into(),
        ];
        for x in self.groups {
            out.push(x.into());
        }
        out
    }
}

#[derive(Debug)]
enum Event {
    Start { kind: GroupType, join: bool },
    End { linecol: LineCol, off: usize },
    Token(Token),
    Error(ParseError),
}

fn next_tk(src: &[Token], skip: bool) -> &[Token] {
    if skip && src[0].kind() == TokenKind::Whitespace {
        &src[1..]
    } else {
        src
    }
}
