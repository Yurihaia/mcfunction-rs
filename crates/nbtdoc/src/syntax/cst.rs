use super::{group::NdGroupType, tokens::NdTokenKind, NbtdocLang};
use mcfunction_parse::ast::{Ast, AstView, CstNode, OwnedNode, SyntaxKind};
use std::sync::Arc;

pub type Node = OwnedNode<Arc<Ast<Arc<str>, NbtdocLang>>>;
pub type RefNode<'a> = OwnedNode<DerefRef<'a, Arc<Ast<Arc<str>, NbtdocLang>>>>;

// Helper trait for ownership stuff
pub trait NH {
    fn into_arc(self) -> Node;
    fn as_ref(&self) -> RefNode;
    fn view(&self) -> AstView<Arc<str>, NbtdocLang>;
}
impl NH for Node {
    fn into_arc(self) -> Node {
        self
    }

    fn as_ref(&self) -> RefNode {
        self.borrow(|v| DerefRef(v))
    }

    fn view(&self) -> AstView<Arc<str>, NbtdocLang> {
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

    fn view(&self) -> AstView<Arc<str>, NbtdocLang> {
        self.view()
    }
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
            type Language = NbtdocLang;
            type Node = N;

            fn can_cast(value: AstView<Arc<str>, NbtdocLang>) -> bool {
                match value.kind() {
                    SyntaxKind::Group(NdGroupType::$ty) => true,
                    _ => false,
                }
            }

            fn view(&self) -> AstView<Arc<str>, NbtdocLang> {
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
                Self(self.0.into_arc())
            }

            pub fn as_ref(&self) -> $id<RefNode> {
                Self(self.0.as_ref())
            }
        }
    };
    ($id:ident, joined $ty:ident) => {
        impl<N: NH> CstNode for $id<N> {
            type String = Arc<str>;
            type Language = NbtdocLang;
            type Node = N;

            fn can_cast(value: AstView<Arc<str>, NbtdocLang>) -> bool {
                match value.kind() {
                    SyntaxKind::Joined(NdGroupType::$ty) => true,
                    _ => false,
                }
            }

            fn view(&self) -> AstView<Arc<str>, NbtdocLang> {
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
            pub fn string(&self) -> &str {
                self.0.view().string()
            }

            pub fn into_arc(self) -> $id<Node> {
                Self(self.0.into_arc())
            }

            pub fn as_ref(&self) -> $id<RefNode> {
                Self(self.0.as_ref())
            }
        }
    };
    ($id:ident, token $ty:ident) => {
        impl<N: NH> CstNode for $id<N> {
            type String = Arc<str>;
            type Language = NbtdocLang;
            type Node = N;

            fn can_cast(value: AstView<Arc<str>, NbtdocLang>) -> bool {
                match value.kind() {
                    SyntaxKind::Token(NdTokenKind::$ty) => true,
                    _ => false,
                }
            }

            fn view(&self) -> AstView<Arc<str>, NbtdocLang> {
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
                Self(self.0.into_arc())
            }

            #[allow(dead_code)]
            pub fn as_ref(&self) -> $id<RefNode> {
                Self(self.0.as_ref())
            }
        }
    };
    (enum $id:ident, $( $nid:ident $hty:ident $xid:ident $($gfn:ident $ret:ident)? ),+ ) => {
        impl<N: NH> CstNode for $id<N> {
            type String = Arc<str>;
            type Language = NbtdocLang;
            type Node = N;

            fn can_cast(value: AstView<Arc<str>, NbtdocLang>) -> bool {
                match value.kind() {
                    $(cst_macro_helper!($hty $xid) => true),+,
                    _ => false
                }
            }

            fn view(&self) -> AstView<Arc<str>, NbtdocLang> {
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
        SyntaxKind::Token(NdTokenKind::$id)
    };
    (group $id:ident) => {
        SyntaxKind::Group(NdGroupType::$id)
    };
}

#[derive(Debug, PartialEq, Eq)]
pub struct File<N>(N);
impl_cst!(File, group File);
impl<N: NH> File<N> {
    pub fn items(&self) -> impl Iterator<Item = Item<RefNode>> {
        self.0.as_ref().children()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Item<N>(N);
impl_cst!(Item, group Item);
impl<N: NH> Item<N> {
    pub fn doc_comments(&self) -> Option<DocCommentGroup<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn compound(&self) -> Option<Compound<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn enum_def(&self) -> Option<Enum<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn mod_decl(&self) -> Option<Mod<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn use_decl(&self) -> Option<Use<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn compound_inject(&self) -> Option<CompoundInject<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn enum_inject(&self) -> Option<EnumInject<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn describes(&self) -> Option<Describes<RefNode>> {
        self.0.as_ref().first_child()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Compound<N>(N);
impl_cst!(Compound, group CompoundDef);
impl<N: NH> Compound<N> {
    pub fn name(&self) -> Option<Ident<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn extends(&self) -> Option<CompoundExtends<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn fields(&self) -> impl Iterator<Item = CompoundField<RefNode>> {
        self.0.as_ref().children()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct CompoundExtends<N>(N);
impl_cst!(CompoundExtends, group CompoundExtends);
impl<N: NH> CompoundExtends<N> {
    pub fn ident_path(&self) -> Option<IdentPath<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn registry_index(&self) -> Option<RegistryIndex<RefNode>> {
        self.0.as_ref().first_child()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct CompoundField<N>(N);
impl_cst!(CompoundField, group CompoundField);
impl<N: NH> CompoundField<N> {
    pub fn doc_comments(&self) -> Option<DocCommentGroup<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn name(&self) -> Option<IdentOrString<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn field_type(&self) -> Option<FieldType<RefNode>> {
        self.0.as_ref().first_child()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum FieldType<N> {
    Scalar(N),
    Array(N),
    List(N),
    Id(N),
    Union(N),
    Named(N),
    Index(N),
}
impl_cst!(enum FieldType,
    Scalar group ScalarType scalar ScalarType,
    Array group ArrayType array ArrayType,
    List group ListType list ListType,
    Id group IdType id IdType,
    Union group UnionType union UnionType,
    Named group NamedType named NamedType,
    Index group IndexType index IndexType
);
#[derive(Debug, PartialEq, Eq)]
pub struct ScalarType<N>(N);
impl_cst!(ScalarType, group ScalarType);
impl<N: NH> ScalarType<N> {
    pub fn ty(&self) -> Option<Primitive<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn range(&self) -> Option<Range<RefNode>> {
        self.0.as_ref().first_child()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum Primitive<N> {
    Boolean(N),
    Byte(N),
    Short(N),
    Int(N),
    Long(N),
    Float(N),
    Double(N),
    String(N),
}
impl_cst!(enum Primitive,
    Boolean token BooleanKw,
    Byte token ByteKw,
    Short token ShortKw,
    Int token IntKw,
    Long token LongKw,
    Float token FloatKw,
    Double token DoubleKw,
    String token StringKw
);
#[derive(Debug, PartialEq, Eq)]
pub struct ArrayType<N>(N);
impl_cst!(ArrayType, group ArrayType);
impl<N: NH> ArrayType<N> {
    pub fn ty(&self) -> Option<Primitive<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn value_range(&self) -> Option<Range<RefNode>> {
        self.0
            .as_ref()
            .first_child::<ArrayBracket<RefNode>>()?
            .into_node()
            .prev_sibling()
    }

    pub fn len_range(&self) -> Option<Range<RefNode>> {
        self.0
            .as_ref()
            .first_child::<ArrayBracket<RefNode>>()?
            .into_node()
            .next_sibling()
    }
}

// Internal
struct ArrayBracket<N>(N);
impl_cst!(ArrayBracket, token LBracket);
#[derive(Debug, PartialEq, Eq)]
pub struct ListType<N>(N);
impl_cst!(ListType, group ListType);
impl<N: NH> ListType<N> {
    pub fn ty(&self) -> Option<FieldType<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn len_range(&self) -> Option<Range<RefNode>> {
        self.0.as_ref().first_child()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct IdType<N>(N);
impl_cst!(IdType, group IdType);
impl<N: NH> IdType<N> {
    pub fn registry(&self) -> Option<MinecraftIdent<RefNode>> {
        self.0.as_ref().first_child()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct UnionType<N>(N);
impl_cst!(UnionType, group UnionType);
impl<N: NH> UnionType<N> {
    pub fn types(&self) -> impl Iterator<Item = FieldPath<RefNode>> {
        self.0.as_ref().children()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct NamedType<N>(N);
impl_cst!(NamedType, group NamedType);
impl<N: NH> NamedType<N> {
    pub fn name(&self) -> Option<IdentPath<RefNode>> {
        self.0.as_ref().first_child()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct IndexType<N>(N);
impl_cst!(IndexType, group IndexType);
impl<N: NH> IndexType<N> {
    pub fn index(&self) -> Option<RegistryIndex<RefNode>> {
        self.0.as_ref().first_child()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Enum<N>(N);
impl_cst!(Enum, group EnumDef);
impl<N: NH> Enum<N> {
    pub fn ty(&self) -> Option<Primitive<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn name(&self) -> Option<Ident<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn entries(&self) -> impl Iterator<Item = EnumEntry<RefNode>> {
        self.0.as_ref().children()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct EnumEntry<N>(N);
impl_cst!(EnumEntry, group EnumEntry);
impl<N: NH> EnumEntry<N> {
    pub fn doc_comments(&self) -> Option<DocCommentGroup<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn name(&self) -> Option<Ident<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn value(&self) -> Option<EnumValue<RefNode>> {
        self.0.as_ref().first_child()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum EnumValue<N> {
    Float(N),
    String(N),
}
impl_cst!(enum EnumValue,
    Float token Float,
    String token QuotedString
);
#[derive(Debug, PartialEq, Eq)]
pub struct Ident<N>(N);
impl_cst!(Ident, token Ident);
#[derive(Debug, PartialEq, Eq)]
pub struct EnumInject<N>(N);
impl_cst!(EnumInject, group EnumInject);
impl<N: NH> EnumInject<N> {
    pub fn ty(&self) -> Option<Primitive<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn target(&self) -> Option<IdentPath<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn entries(&self) -> impl Iterator<Item = EnumEntry<RefNode>> {
        self.0.as_ref().children()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct CompoundInject<N>(N);
impl_cst!(CompoundInject, group CompoundInject);
impl<N: NH> CompoundInject<N> {
    pub fn target(&self) -> Option<IdentPath<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn entries(&self) -> impl Iterator<Item = CompoundField<RefNode>> {
        self.0.as_ref().children()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Mod<N>(N);
impl_cst!(Mod, group ModDecl);
impl<N: NH> Mod<N> {
    pub fn name(&self) -> Option<Ident<RefNode>> {
        self.0.as_ref().first_child()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Use<N>(N);
impl_cst!(Use, group UseStatement);
impl<N: NH> Use<N> {
    pub fn export(&self) -> bool {
        self.0.as_ref().first_child::<Export<RefNode>>().is_some()
    }

    pub fn path(&self) -> Option<IdentPath<RefNode>> {
        self.0.as_ref().first_child()
    }
}

// Internal
struct Export<N>(N);
impl_cst!(Export, token ExportKw);
#[derive(Debug, PartialEq, Eq)]
pub struct Describes<N>(N);
impl_cst!(Describes, group DescribesStatement);
impl<N: NH> Describes<N> {
    pub fn compound(&self) -> Option<IdentPath<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn registry(&self) -> Option<MinecraftIdent<RefNode>> {
        self.0.as_ref().first_child()
    }

    pub fn targets(&self) -> Option<DescribesTargets<RefNode>> {
        self.0.as_ref().first_child()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct DescribesTargets<N>(N);
impl_cst!(DescribesTargets, group DescribesBody);
impl<N: NH> DescribesTargets<N> {
    pub fn ids(&self) -> impl Iterator<Item = MinecraftIdent<RefNode>> {
        self.0.as_ref().children()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct IdentPath<N>(N);
impl_cst!(IdentPath, group IdentPath);
impl<N: NH> IdentPath<N> {
    pub fn segments(&self) -> impl Iterator<Item = IdentPathSegment<RefNode>> + '_ {
        self.0.as_ref().children()
    }

    pub fn root(&self) -> bool {
        if let Some(child) = self.0.view().first_child() {
            match child.kind() {
                SyntaxKind::Token(NdTokenKind::ColonColon) => true,
                _ => false,
            }
        } else {
            false
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum IdentPathSegment<N> {
    Ident(N),
    Super(N),
}
impl_cst!(enum IdentPathSegment,
    Ident token Ident,
    Super token SuperKw
);
impl<N: NH> IdentPathSegment<N> {
    pub fn ident(&self) -> Option<&str> {
        match self {
            Self::Ident(v) => Some(v.view().string()),
            _ => None,
        }
    }

    pub fn is_super(&self) -> bool {
        match self {
            Self::Super(_) => true,
            _ => false,
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct RegistryIndex<N>(N);
impl_cst!(RegistryIndex, group RegistryIndex);
impl<N: NH> RegistryIndex<N> {
    pub fn registry(&self) -> Option<MinecraftIdent<RefNode>> {
        self.0.as_ref().first_child()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct FieldPath<N>(N);
impl_cst!(FieldPath, group FieldPath);
impl<N: NH> FieldPath<N> {
    pub fn segments(&self) -> impl Iterator<Item = FieldPathSegment<RefNode>> + '_ {
        self.0.as_ref().children()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub enum FieldPathSegment<N> {
    Ident(N),
    Super(N),
    QuotedString(N),
}
impl_cst!(enum FieldPathSegment,
    Ident token Ident,
    Super token SuperKw,
    QuotedString token QuotedString
);
impl<N: NH> FieldPathSegment<N> {
    pub fn ident(&self) -> Option<&str> {
        match self {
            Self::Ident(v) => Some(v.view().string()),
            _ => None,
        }
    }

    pub fn string(&self) -> Option<&str> {
        match self {
            Self::QuotedString(v) => Some(v.view().string()),
            _ => None,
        }
    }

    pub fn is_super(&self) -> bool {
        match self {
            Self::Super(_) => true,
            _ => false,
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct MinecraftIdent<N>(N);
impl_cst!(MinecraftIdent, joined MinecraftIdent);
#[derive(Debug, PartialEq, Eq)]
pub enum IdentOrString<N> {
    Ident(N),
    QuotedString(N),
}
impl_cst!(enum IdentOrString,
    Ident token Ident,
    QuotedString token QuotedString
);
impl<N: NH> IdentOrString<N> {
    pub fn ident(&self) -> Option<&str> {
        match self {
            Self::Ident(v) => Some(v.view().string()),
            _ => None,
        }
    }

    pub fn string(&self) -> Option<&str> {
        match self {
            Self::QuotedString(v) => Some(v.view().string()),
            _ => None,
        }
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct Range<N>(N);
impl_cst!(Range, group Range);
impl<N: NH> Range<N> {
    pub fn lower(&self) -> Option<Float<RefNode>> {
        self.0.as_ref().first_child::<DDorFloat<RefNode>>()?.float()
    }

    pub fn upper(&self) -> Option<Float<RefNode>> {
        self.0.as_ref().last_child::<DDorFloat<RefNode>>()?.float()
    }

    pub fn single(&self) -> bool {
        self.0.as_ref().first_child::<DotDot<RefNode>>().is_none()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct DocCommentGroup<N>(N);
impl_cst!(DocCommentGroup, group DocCommentGroup);
impl<N: NH> DocCommentGroup<N> {
    pub fn comments(&self) -> impl Iterator<Item = DocComment<RefNode>> {
        self.0.as_ref().children()
    }
}
#[derive(Debug, PartialEq, Eq)]
pub struct DocComment<N>(N);
impl_cst!(DocComment, token DocComment);
#[derive(Debug, PartialEq, Eq)]
pub struct Float<N>(N);
impl_cst!(Float, token Float);

// Internal
struct DotDot<N>(N);
impl_cst!(DotDot, token DotDot);

// Internal
enum DDorFloat<N> {
    DotDot(N),
    Float(N),
}
impl_cst!(enum DDorFloat,
    DotDot token DotDot,
    Float token Float float Float
);
