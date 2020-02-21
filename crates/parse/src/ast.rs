use crate::parser::Event;
use crate::{parser::Language, LineCol, ParseError, Span};

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum SyntaxKind<L: Language> {
    Root,
    Group(L::GroupType),
    Joined(L::GroupType),
    Token(L::TokenKind),
    Error(ParseError<L>),
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub struct Ast<T: AsRef<str>, L: Language> {
    src: T,
    arena: Vec<AstNode<L>>,
    errors: Vec<usize>,
}

impl<T: AsRef<str>, L: Language> Ast<T, L> {
    pub fn root(&self) -> AstView<T, L> {
        AstView(0, self)
    }

    pub fn errors(&self) -> impl Iterator<Item = AstView<T, L>> + '_ {
        self.errors.iter().copied().map(move |v| AstView(v, self))
    }

    pub fn bind(&self, view: UnboundView<L>) -> AstView<T, L> {
        AstView(view.0, self)
    }

    pub fn retype_src<I: From<T> + AsRef<str>>(self) -> Ast<I, L> {
        self.retype_src_with(I::from)
    }

    pub fn retype_src_with<F: FnOnce(T) -> I, I: AsRef<str>>(self, f: F) -> Ast<I, L> {
        Ast {
            arena: self.arena,
            errors: self.errors,
            src: f(self.src),
        }
    }
}

#[derive(Debug, PartialEq, Eq, Clone)]
pub(crate) struct AstNode<L: Language> {
    kind: SyntaxKind<L>,
    string: (usize, usize),
    span: Span,
    children: Vec<usize>,
    parent: Option<usize>,
    sibling_index: usize,
}

pub struct AstView<'a, T: AsRef<str>, L: Language>(usize, &'a Ast<T, L>);

impl<T: AsRef<str>, L: Language> Clone for AstView<'_, T, L> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<T: AsRef<str>, L: Language> Copy for AstView<'_, T, L> {}

pub struct UnboundView<L: Language>(usize, std::marker::PhantomData<fn() -> L>);

impl<L: Language> Clone for UnboundView<L> {
    fn clone(&self) -> Self {
        *self
    }
}

impl<L: Language> Copy for UnboundView<L> {}

impl<'a, T: AsRef<str>, L: Language + std::fmt::Debug> std::fmt::Debug for AstView<'a, T, L> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.node())
    }
}

impl<'a, T: AsRef<str>, L: Language> PartialEq for AstView<'a, T, L> {
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && std::ptr::eq(self.1, other.1)
    }
}

impl<T: AsRef<str>, L: Language> Eq for AstView<'_, T, L> {}

impl<'a, T: AsRef<str>, L: Language> AstView<'a, T, L> {
    pub fn kind(&self) -> &SyntaxKind<L> {
        &self.node().kind
    }

    pub fn string(&self) -> &'a str {
        let (start, end) = self.node().string;
        &self.1.src.as_ref()[start..end]
    }

    pub fn span(&self) -> Span {
        self.node().span
    }

    pub fn unbind(&self) -> UnboundView<L> {
        UnboundView(self.0, std::marker::PhantomData)
    }

    pub fn parent(self) -> Option<Self> {
        self.node().parent.map(|v| self.new(v))
    }

    pub fn children(&self) -> impl Iterator<Item = AstView<'_, T, L>> {
        self.node().children.iter().map(move |v| self.new(*v))
    }

    pub fn first_child(self) -> Option<Self> {
        self.node().children.first().map(|v| self.new(*v))
    }

    pub fn last_child(self) -> Option<Self> {
        self.node().children.last().map(|v| self.new(*v))
    }

    pub fn nth_child(self, n: usize) -> Option<Self> {
        self.node().children.get(n).map(|v| self.new(*v))
    }

    pub fn next_sibling(self) -> Option<Self> {
        self.nth_next_sibling(1)
    }

    pub fn prev_sibling(self) -> Option<Self> {
        self.nth_prev_sibling(1)
    }

    pub fn nth_next_sibling(self, n: usize) -> Option<Self> {
        self.parent()?
            .nth_child(self.node().sibling_index.checked_add(n)?)
    }

    pub fn nth_prev_sibling(self, n: usize) -> Option<Self> {
        self.parent()?
            .nth_child(self.node().sibling_index.checked_sub(n)?)
    }

    fn new(&self, idx: usize) -> Self {
        AstView(idx, self.1)
    }

    fn node(&self) -> &AstNode<L> {
        &self.1.arena[self.0]
    }
}

pub fn build_ast<T: AsRef<str>, L: Language>(
    events: Vec<Event<L>>,
    src: T,
    save_errors: bool,
) -> Ast<T, L> {
    let mut out = Ast {
        arena: vec![AstNode {
            kind: SyntaxKind::Root,
            children: vec![],
            parent: None,
            sibling_index: 0,
            span: Span::default(),
            string: (0, 0),
        }],
        errors: vec![],
        src,
    };
    let mut errors = vec![];
    let mut ind_stack = vec![(0, None)];
    for evt in events {
        match evt {
            Event::Start { kind, join } => {
                let ind = out.arena.len();
                let parent = ind_stack.last().unwrap().0;
                out.arena.push(AstNode {
                    kind: if join {
                        SyntaxKind::Joined(kind)
                    } else {
                        SyntaxKind::Group(kind)
                    },
                    children: vec![],
                    parent: Some(parent),
                    sibling_index: out.arena[parent].children.len(),
                    span: Span::new(LineCol::new(0, 0), LineCol::new(0, 0)),
                    string: (0, 0),
                });
                out.arena[parent].children.push(ind);
                ind_stack.push((ind, None));
            }
            Event::End { linecol, off } => {
                let (node, span) = ind_stack.pop().unwrap();
                let (span, start, end) = span.unwrap_or((Span::new(linecol, linecol), off, off));
                let parent = ind_stack.last_mut().unwrap();
                match &mut parent.1 {
                    Some((sp, s, e)) => {
                        *sp = sp.union(&span);
                        *s = (*s).min(start);
                        *e = (*e).max(end);
                    }
                    v => *v = Some((span, start, end)),
                }
                out.arena[node].string = (start, end);
                out.arena[node].span = span;
            }
            Event::Token(tk) => {
                let parent = ind_stack.last_mut().unwrap();
                match &mut parent.1 {
                    Some((sp, s, e)) => {
                        *sp = sp.union(&tk.span());
                        *s = (*s).min(tk.start());
                        *e = (*e).max(tk.end());
                    }
                    v @ None => *v = Some((tk.span(), tk.start(), tk.end())),
                }
                let cind = out.arena[parent.0].children.len();
                let ind = out.arena.len();
                out.arena.push(AstNode {
                    kind: SyntaxKind::Token(tk.kind()),
                    children: vec![],
                    parent: Some(parent.0),
                    sibling_index: cind,
                    span: tk.span(),
                    string: (tk.start(), tk.end()),
                });
                out.arena[parent.0].children.push(ind);
            }
            Event::Error(err) => {
                let parent = ind_stack.last_mut().unwrap();
                let ind = out.arena.len();
                let cind = out.arena[parent.0].children.len();
                out.arena.push(AstNode {
                    kind: SyntaxKind::Error(err),
                    children: vec![],
                    parent: Some(parent.0),
                    sibling_index: cind,
                    span: Span::default(),
                    string: (0, 0),
                });
                errors.push(ind);
                out.arena[ind_stack.last().unwrap().0].children.push(ind);
            }
        }
    }

    'out: for err in &errors {
        let mut view = AstView(*err, &out);
        while let SyntaxKind::Error(_) = view.kind() {
            view = match view.next_sibling() {
                Some(v) => v,
                None => {
                    view = view.parent().unwrap();
                    let ind = view.0;
                    out.arena[*err].span =
                        Span::new(out.arena[ind].span.end(), out.arena[ind].span.end());
                    out.arena[*err].string = out.arena[ind].string;
                    continue 'out;
                }
            };
        }
        let ind = view.0;
        out.arena[*err].span = out.arena[ind].span;
        out.arena[*err].string = out.arena[ind].string;
    }
    if save_errors {
        out.errors = errors;
    }
    out
}
