use super::{McGroupType, McTokenKind, McfLang};
use mcfunction_parse::ast::{Ast, AstView, CstNode, OwnedNode, SyntaxKind};
use std::sync::Arc;
use util::commands::{CommandNodeType, Commands, Index, ParserType};

pub type Node = OwnedNode<Arc<Ast<Arc<str>, McfLang>>>;
pub type RefNode<'a> = OwnedNode<DerefRef<'a, Arc<Ast<Arc<str>, McfLang>>>>;

pub trait NH {
    fn into_arc(self) -> Node;
    fn as_ref(&self) -> RefNode;
    fn view(&self) -> AstView<Arc<str>, McfLang>;
}
impl NH for Node {
    fn into_arc(self) -> Node {
        self
    }

    fn as_ref(&self) -> RefNode {
        self.borrow(|v| DerefRef(v))
    }

    fn view(&self) -> AstView<Arc<str>, McfLang> {
        self.view()
    }
}
impl<'a> NH for RefNode<'a> {
    fn into_arc(self) -> Node {
        self.convert(|d| d.0.clone())
    }

    fn as_ref(&self) -> RefNode {
        self.borrow(|d| *d)
    }

    fn view(&self) -> AstView<Arc<str>, McfLang> {
        self.view()
    }
}

pub trait CommandNodeItem: CstNode<String = Arc<str>, Language = McfLang> {
    fn valid_type(pt: ParserType) -> bool;
    fn is_literal() -> bool;
}

#[derive(Debug, PartialEq, Eq)]
pub struct DerefRef<'a, D>(&'a D);
impl<'a, D> std::ops::Deref for DerefRef<'a, D>
where
    D: std::ops::Deref,
{
    type Target = D::Target;
    fn deref(&self) -> &Self::Target {
        self.0.deref()
    }
}
impl<'a, T> Clone for DerefRef<'a, T> {
    fn clone(&self) -> Self {
        *self
    }
}
impl<'a, T> Copy for DerefRef<'a, T> {}

macro_rules! impl_cst {
    ($id:ident, group $ty:ident) => {
        impl<N: NH> CstNode for $id<N> {
            type String = Arc<str>;
            type Language = McfLang;
            type Node = N;

            fn can_cast(value: AstView<Arc<str>, McfLang>) -> bool {
                match value.kind() {
                    SyntaxKind::Group(McGroupType::$ty) => true,
                    _ => false,
                }
            }

            fn view(&self) -> AstView<Arc<str>, McfLang> {
                self.0.view()
            }

            fn into_node(self) -> Self::Node {
                self.0
            }

            fn new(node: N) -> Self {
                Self(node)
            }
        }

        impl<N: NH> $id<N> {
            pub fn into_arc(self) -> $id<Node> {
                $id(self.0.into_arc())
            }

            pub fn as_ref(&self) -> $id<RefNode> {
                $id(self.0.as_ref())
            }
        }
    };
    ($id:ident, joined $ty:ident) => {
        impl<N: NH> CstNode for $id<N> {
            type String = Arc<str>;
            type Language = McfLang;
            type Node = N;

            fn can_cast(value: AstView<Arc<str>, McfLang>) -> bool {
                match value.kind() {
                    SyntaxKind::Joined(McGroupType::$ty) => true,
                    _ => false,
                }
            }

            fn view(&self) -> AstView<Arc<str>, McfLang> {
                self.0.view()
            }

            fn into_node(self) -> Self::Node {
                self.0
            }

            fn new(node: N) -> Self {
                Self(node)
            }
        }

        #[allow(dead_code)]
        impl<N: NH> $id<N> {
            pub fn string(&self) -> &str {
                self.0.view().string()
            }

            pub fn into_arc(self) -> $id<Node> {
                $id(self.0.into_arc())
            }

            pub fn as_ref(&self) -> $id<RefNode> {
                $id(self.0.as_ref())
            }
        }
    };
    ($id:ident, token $ty:ident) => {
        impl<N: NH> CstNode for $id<N> {
            type String = Arc<str>;
            type Language = McfLang;
            type Node = N;

            fn can_cast(value: AstView<Arc<str>, McfLang>) -> bool {
                match value.kind() {
                    SyntaxKind::Token(McTokenKind::$ty) => true,
                    _ => false,
                }
            }

            fn view(&self) -> AstView<Arc<str>, McfLang> {
                self.0.view()
            }

            fn into_node(self) -> Self::Node {
                self.0
            }

            fn new(node: N) -> Self {
                Self(node)
            }
        }

        impl<N: NH> $id<N> {
            #[allow(dead_code)]
            pub fn into_arc(self) -> $id<Node> {
                $id(self.0.into_arc())
            }

            #[allow(dead_code)]
            pub fn as_ref(&self) -> $id<RefNode> {
                $id(self.0.as_ref())
            }
        }
    };
    (enum $id:ident, $( $nid:ident $hty:ident $xid:ident $($gfn:ident $ret:ident)? ),+ ) => {
        impl<N: NH> CstNode for $id<N> {
            type String = Arc<str>;
            type Language = McfLang;
            type Node = N;

            fn can_cast(value: AstView<Arc<str>, McfLang>) -> bool {
                match value.kind() {
                    $(cst_macro_helper!($hty $xid) => true),+,
                    _ => false
                }
            }

            fn view(&self) -> AstView<Arc<str>, McfLang> {
                match self {
                    $(Self::$nid(v) => v.view()),*
                }
            }

            fn into_node(self) -> Self::Node {
                match self {
                    $(Self::$nid(v) => v),*
                }
            }

            fn new(node: N) -> Self {
                match node.view().kind() {
                    $(cst_macro_helper!($hty $xid) => Self::$nid(node)),*,
                    _ => panic!("Invalid node type")
                }
            }
        }

        impl<N: NH> $id<N> {
            $( $(
                pub fn $gfn(self) -> Option<$ret<N>> {
                    match self {
                        Self::$nid(v) => if $ret::<N>::can_cast(v.view()) {
                            Some($ret::new(v))
                        } else {
                            None
                        },
                        _ => None
                    }
                }
            )? )+

            #[allow(dead_code)]
            pub fn into_arc(self) -> $id<Node> {
                $id::<Node>::new(self.into_node().into_arc())
            }

            #[allow(dead_code)]
            pub fn as_ref(&self) -> $id<RefNode> {
                $id::<RefNode>::new(match self {
                    $( Self::$nid(v) => v.as_ref() ),*
                })
            }
        }
    }
}
macro_rules! cst_macro_helper {
    (token $id:ident) => {
        SyntaxKind::Token(McTokenKind::$id)
    };
    (group $id:ident) => {
        SyntaxKind::Group(McGroupType::$id)
    };
    (joined $id:ident) => {
        SyntaxKind::Joined(McGroupType::$id)
    };
}
macro_rules! impl_cni {
    ($id:ident $ty:ident {}) => {
        impl<N: NH> CommandNodeItem for $id<N> {
            fn valid_type(pt: ParserType) -> bool {
                match pt {
                    ParserType::$ty { .. } => true,
                    _ => false,
                }
            }

            fn is_literal() -> bool {
                false
            }
        }
    };
    ($id:ident $ty:ident) => {
        impl<N: NH> CommandNodeItem for $id<N> {
            fn valid_type(pt: ParserType) -> bool {
                match pt {
                    ParserType::$ty => true,
                    _ => false,
                }
            }

            fn is_literal() -> bool {
                false
            }
        }
    };
}

#[derive(Debug, PartialEq, Eq)]
pub struct File<N>(N);
impl_cst!(File, group File);
impl<N: NH> File<N> {
    pub fn lines(&self) -> impl Iterator<Item = Line<RefNode>> {
        self.0.as_ref().children()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub enum Line<N> {
    Command(N),
    Comment(N),
}
impl_cst!(enum Line,
    Command group Command command Command,
    Comment group Comment comment Comment
);

#[derive(Debug, PartialEq, Eq)]
pub struct Command<N>(N);
impl_cst!(Command, group Command);
impl<N: NH> Command<N> {
    pub fn nodes(&self) -> impl Iterator<Item = CommandNode<RefNode>> {
        self.0.as_ref().children()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct CommandNode<N>(Index, N);
impl<N: NH> CstNode for CommandNode<N> {
    type String = Arc<str>;
    type Language = McfLang;
    type Node = N;
    fn view(&self) -> AstView<Self::String, Self::Language> {
        self.1.view()
    }
    fn can_cast(view: AstView<Self::String, Self::Language>) -> bool {
        match view.kind() {
            SyntaxKind::Group(McGroupType::CommandNode(_)) => true,
            _ => false,
        }
    }
    fn into_node(self) -> Self::Node {
        self.1
    }
    fn new(node: Self::Node) -> Self {
        Self(
            match node.view().kind() {
                SyntaxKind::Group(McGroupType::CommandNode(ind)) => *ind,
                _ => panic!("Invalid CST cast"),
            },
            node,
        )
    }
}
impl<N: NH> CommandNode<N> {
    pub fn index(&self) -> Index {
        self.0
    }

    pub fn child_type<'a, T: CommandNodeItem<Node = RefNode<'a>>>(
        &'a self,
        cmds: &Commands,
    ) -> Option<T> {
        let command = &cmds[self.0];
        if match command.node_type() {
            CommandNodeType::Literal => {
                T::is_literal() && self.1.view().first_child()?.string() == command.name()
            }
            CommandNodeType::Argument { parser_type } => T::valid_type(parser_type),
            _ => false,
        } {
            if !T::can_cast(self.1.view().first_child()?) {
                return None;
            }
            self.1.as_ref().first_child()
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct Comment<N>(N);
impl_cst!(Comment, group Comment);

#[derive(Debug, PartialEq, Eq)]
pub enum NbtValue<N> {
    Compound(N),
    Sequence(N),
    Number(N),
    String(N),
    Boolean(N),
}
impl_cst!(enum NbtValue,
    Compound group NbtCompound compound NbtCompound,
    Sequence group NbtSequence sequence NbtSequence,
    Number group NbtNumber number NbtNumber,
    String group NbtString string NbtString,
    Boolean group NbtBoolean boolean NbtBoolean
);
impl_cni!(NbtValue NbtTag);

#[derive(Debug, PartialEq, Eq)]
pub struct NbtCompound<N>(N);
impl_cst!(NbtCompound, group NbtCompound);
impl_cni!(NbtCompound NbtCompoundTag);
impl<N: NH> NbtCompound<N> {
    pub fn entries(&self) -> impl Iterator<Item = NbtCompoundEntry<RefNode>> {
        self.0.as_ref().children()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NbtCompoundEntry<N>(N);
impl_cst!(NbtCompoundEntry, group NbtCompoundEntry);
impl<N: NH> NbtCompoundEntry<N> {
    pub fn key(&self) -> Option<McfString<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn value(&self) -> Option<NbtValue<RefNode>> {
        self.0.as_ref().first_child()
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NbtSequence<N>(N);
impl_cst!(NbtSequence, group NbtSequence);
impl<N: NH> NbtSequence<N> {
    pub fn seq_type(&self) -> NbtSequenceType {
        let semi = self.0.as_ref().first_child::<Semicolon<RefNode>>();
        match semi {
            Some(semi) => {
                if semi
                    .0
                    .as_ref()
                    .prev_sibling::<NbtPrefixB<RefNode>>()
                    .is_some()
                {
                    NbtSequenceType::ByteArray
                } else if semi
                    .0
                    .as_ref()
                    .prev_sibling::<NbtPrefixI<RefNode>>()
                    .is_some()
                {
                    NbtSequenceType::IntArray
                } else if semi
                    .0
                    .as_ref()
                    .prev_sibling::<NbtPrefixL<RefNode>>()
                    .is_some()
                {
                    NbtSequenceType::LongArray
                } else {
                    NbtSequenceType::ErrorArray
                }
            }
            None => NbtSequenceType::List,
        }
    }

    pub fn entries(&self) -> impl Iterator<Item = NbtValue<RefNode>> {
        self.0.as_ref().children()
    }
}

#[derive(Copy, Clone, Debug, PartialEq, Eq)]
pub enum NbtSequenceType {
    List,
    ByteArray,
    IntArray,
    LongArray,
    ErrorArray,
}

// INTERNAL USE
struct Semicolon<N>(N);
impl_cst!(Semicolon, token Semicolon);

// INTERNAL USE
struct NbtPrefixB<N>(N);
impl_cst!(NbtPrefixB, joined NbtPrefixB);

// INTERNAL USE
struct NbtPrefixI<N>(N);
impl_cst!(NbtPrefixI, joined NbtPrefixI);

// INTERNAL USE
struct NbtPrefixL<N>(N);
impl_cst!(NbtPrefixL, joined NbtPrefixL);

// INTERNAL USE
struct NbtSuffixB<N>(N);
impl_cst!(NbtSuffixB, joined NbtSuffixB);

// INTERNAL USE
struct NbtSuffixS<N>(N);
impl_cst!(NbtSuffixS, joined NbtSuffixS);

// INTERNAL USE
struct NbtSuffixL<N>(N);
impl_cst!(NbtSuffixL, joined NbtSuffixL);

// INTERNAL USE
struct NbtSuffixF<N>(N);
impl_cst!(NbtSuffixF, joined NbtSuffixF);

// INTERNAL USE
struct NbtSuffixD<N>(N);
impl_cst!(NbtSuffixD, joined NbtSuffixD);

#[derive(Debug, PartialEq, Eq)]
pub struct NbtNumber<N>(N);
impl_cst!(NbtNumber, group NbtNumber);
impl<N: NH> NbtNumber<N> {
    pub fn byte(&self) -> Option<FloatToken<RefNode>> {
        if self
            .0
            .as_ref()
            .last_child::<NbtSuffixB<RefNode>>()
            .is_some()
        {
            self.0.as_ref().first_child()
        } else {
            None
        }
    }

    pub fn short(&self) -> Option<FloatToken<RefNode>> {
        if self
            .0
            .as_ref()
            .last_child::<NbtSuffixS<RefNode>>()
            .is_some()
        {
            self.0.as_ref().first_child()
        } else {
            None
        }
    }

    pub fn untagged(&self) -> Option<FloatToken<RefNode>> {
        if self.0.view().first_child()?.next_sibling().is_none() {
            self.0.as_ref().first_child()
        } else {
            None
        }
    }

    pub fn long(&self) -> Option<FloatToken<RefNode>> {
        if self
            .0
            .as_ref()
            .last_child::<NbtSuffixL<RefNode>>()
            .is_some()
        {
            self.0.as_ref().first_child()
        } else {
            None
        }
    }

    pub fn float(&self) -> Option<FloatToken<RefNode>> {
        if self
            .0
            .as_ref()
            .last_child::<NbtSuffixF<RefNode>>()
            .is_some()
        {
            self.0.as_ref().first_child()
        } else {
            None
        }
    }

    pub fn double(&self) -> Option<FloatToken<RefNode>> {
        if self
            .0
            .as_ref()
            .last_child::<NbtSuffixD<RefNode>>()
            .is_some()
        {
            self.0.as_ref().first_child()
        } else {
            None
        }
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct NbtString<N>(N);
impl_cst!(NbtString, group NbtString);

#[derive(Debug, PartialEq, Eq)]
pub struct NbtBoolean<N>(N);
impl_cst!(NbtBoolean, group NbtBoolean);
impl<N: NH> NbtNumber<N> {
    pub fn value(&self) -> Option<bool> {
        let id = self.0.view().first_child()?.string();
        Some(match id {
            "true" => true,
            "false" => false,
            _ => return None,
        })
    }
}

#[derive(Debug, PartialEq, Eq)]
pub struct IntRangeArgument<N>(N);
impl_cst!(IntRangeArgument, group Range);
impl_cni!(IntRangeArgument IntRange);

#[derive(Debug, PartialEq, Eq)]
pub enum McfString<N> {
    Quoted(N),
    Unquoted(N),
}
impl_cst!(enum McfString,
    Quoted token QuotedString,
    Unquoted joined UnquotedString
);

#[derive(Debug, PartialEq, Eq)]
pub struct FloatToken<N>(N);
impl_cst!(FloatToken, joined Float);
