use crate::parser::Event;
use crate::{parser::Language, LineCol, ParseError, Span};
use std::convert::From;
use std::{
    iter::{DoubleEndedIterator, ExactSizeIterator},
    ops::Deref,
};

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
    arena: Vec<InnerAstNode<L>>,
    errors: Vec<usize>,
}

impl<T: AsRef<str>, L: Language> Ast<T, L> {
    pub fn root(&self) -> AstView<T, L> {
        AstView(0, self)
    }

    pub fn errors(&self) -> impl Iterator<Item = AstView<T, L>> + '_ {
        self.errors.iter().copied().map(move |v| AstView(v, self))
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

pub trait CstNode {
    type String: AsRef<str>;
    type Language: Language;
    type Node;

    fn view(&self) -> AstView<Self::String, Self::Language>;

    fn can_cast(view: AstView<Self::String, Self::Language>) -> bool;

    fn into_node(self) -> Self::Node;

    fn new(node: Self::Node) -> Self;
}

#[derive(Debug, PartialEq, Eq, Clone)]
struct InnerAstNode<L: Language> {
    kind: SyntaxKind<L>,
    string: (usize, usize),
    span: Span,
    children: Vec<usize>,
    parent: Option<usize>,
    sibling_index: usize,
}

#[derive(Copy, Clone)]
pub struct OwnedNode<D>(usize, D)
where
    D: Deref;

impl<D> OwnedNode<D>
where
    D: Deref,
{
    pub fn convert<F, N>(self, f: F) -> OwnedNode<N>
    where
        F: FnOnce(D) -> N,
        N: Deref,
    {
        OwnedNode(self.0, f(self.1))
    }

    pub fn borrow<'a, F, N>(&'a self, f: F) -> OwnedNode<N>
    where
        F: FnOnce(&'a D) -> N,
        N: Deref + 'a,
    {
        OwnedNode(self.0, f(&self.1))
    }
}

impl<T, L, D> OwnedNode<D>
where
    T: AsRef<str>,
    L: Language,
    D: Deref<Target = Ast<T, L>>,
{
    pub fn from_view(d: D, view: AstView<T, L>) -> Self {
        OwnedNode(view.0, d)
    }

    pub fn view(&self) -> AstView<T, L> {
        AstView(self.0, &*self.1)
    }

    pub fn first_child<C>(self) -> Option<C>
    where
        D: Clone,
        C: CstNode<String = T, Language = L, Node = Self>,
    {
        let id = self.view().children().find(|c| C::can_cast(*c))?.0;
        Some(C::new(OwnedNode(id, self.1.clone())))
    }

    pub fn last_child<C>(self) -> Option<C>
    where
        D: Clone,
        C: CstNode<String = T, Language = L, Node = Self>,
    {
        let id = self.view().children().rev().find(|c| C::can_cast(*c))?.0;
        Some(C::new(OwnedNode(id, self.1.clone())))
    }

    pub fn next_sibling<C>(self) -> Option<C>
    where
        D: Clone,
        C: CstNode<String = T, Language = L, Node = Self>,
    {
        let view = self.view();
        let cind = view.node().sibling_index;
        let id = view.children().skip(cind).find(|c| C::can_cast(*c))?.0;
        Some(C::new(OwnedNode(id, self.1.clone())))
    }

    pub fn prev_sibling<C>(self) -> Option<C>
    where
        D: Clone,
        C: CstNode<String = T, Language = L, Node = Self>,
    {
        let view = self.view();
        let cind = view.node().sibling_index;
        let id = view
            .children()
            .take(cind + 1)
            .rev()
            .find(|c| C::can_cast(*c))?
            .0;
        Some(C::new(OwnedNode(id, self.1.clone())))
    }

    pub fn children<C>(self) -> Children<C, D, T, L>
    where
        D: Clone,
        C: CstNode<String = T, Language = L, Node = Self>,
    {
        Children {
            _pd: std::marker::PhantomData,
            ast: self.1,
            children: self.0,
            index: 0,
        }
    }
}

pub struct Children<C, D, T, L>
where
    C: CstNode,
    T: AsRef<str>,
    L: Language,
    D: Deref<Target = Ast<T, L>> + Clone,
{
    _pd: std::marker::PhantomData<fn() -> C>,
    ast: D,
    children: usize,
    index: usize,
}

impl<C, D, T, L> Iterator for Children<C, D, T, L>
where
    C: CstNode<Node = OwnedNode<D>>,
    T: AsRef<str>,
    L: Language,
    D: Deref<Target = Ast<T, L>> + Clone,
{
    type Item = C;
    fn next(&mut self) -> Option<Self::Item> {
        let child = self.ast.arena[self.children].children.get(self.index)?;
        self.index += 1;
        Some(CstNode::new(OwnedNode(*child, self.ast.clone())))
    }
}

pub struct AstView<'a, T: AsRef<str>, L: Language>(usize, &'a Ast<T, L>);

impl<'a, T, L> AstView<'a, T, L>
where
    T: AsRef<str>,
    L: Language,
{
    pub fn kind<'b>(&'b self) -> &'a SyntaxKind<L> {
        &self.node().kind
    }

    pub fn string<'b>(&'b self) -> &'a str {
        let (start, end) = self.node().string;
        &self.1.src.as_ref()[start..end]
    }

    pub fn span(&self) -> Span {
        self.node().span
    }

    pub fn parent(self) -> Option<Self> {
        self.node().parent.map(|v| self.new(v))
    }

    pub fn children(
        self,
    ) -> impl Iterator<Item = AstView<'a, T, L>> + DoubleEndedIterator + ExactSizeIterator + 'a
    {
        self.node()
            .children
            .clone()
            .into_iter()
            .map(move |v| self.new(v))
    }

    pub fn first_child(self) -> Option<AstView<'a, T, L>> {
        self.node()
            .children
            .first()
            .copied()
            .map(move |v| self.new(v))
    }

    pub fn last_child(self) -> Option<AstView<'a, T, L>> {
        self.node()
            .children
            .last()
            .copied()
            .map(move |v| self.new(v))
    }

    pub fn nth_child(self, n: usize) -> Option<AstView<'a, T, L>> {
        self.node()
            .children
            .get(n)
            .copied()
            .map(move |v| self.new(v))
    }

    pub fn next_sibling(self) -> Option<AstView<'a, T, L>> {
        self.nth_next_sibling(1)
    }

    pub fn prev_sibling(self) -> Option<AstView<'a, T, L>> {
        self.nth_prev_sibling(1)
    }

    pub fn nth_next_sibling(self, n: usize) -> Option<AstView<'a, T, L>> {
        let ind = self.node().sibling_index.checked_add(n)?;
        self.parent()?.nth_child(ind)
    }

    pub fn nth_prev_sibling(self, n: usize) -> Option<AstView<'a, T, L>> {
        let ind = self.node().sibling_index.checked_sub(n)?;
        self.parent()?.nth_child(ind)
    }

    fn new(self, idx: usize) -> AstView<'a, T, L> {
        AstView(idx, self.1)
    }

    fn node(&self) -> &'a InnerAstNode<L> {
        &self.1.arena[self.0]
    }
}

pub fn build_ast<T: AsRef<str>, L: Language>(
    events: Vec<Event<L>>,
    src: T,
    save_errors: bool,
) -> Ast<T, L> {
    let mut out = Ast {
        arena: vec![InnerAstNode {
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
                out.arena.push(InnerAstNode {
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
                out.arena.push(InnerAstNode {
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
                out.arena.push(InnerAstNode {
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

impl<'a, T: AsRef<str>, L: Language> Copy for AstView<'a, T, L> {}
impl<'a, T: AsRef<str>, L: Language> Clone for AstView<'a, T, L> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, T, L> std::fmt::Debug for AstView<'a, T, L>
where
    T: AsRef<str>,
    L: Language + std::fmt::Debug,
{
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self.node())
    }
}
impl<'a, T, L> PartialEq for AstView<'a, T, L>
where
    T: AsRef<str>,
    L: Language,
{
    fn eq(&self, other: &Self) -> bool {
        self.0 == other.0 && std::ptr::eq(self.1.deref(), other.1.deref())
    }
}
impl<'a, T, L> Eq for AstView<'a, T, L>
where
    T: AsRef<str>,
    L: Language,
{
}
