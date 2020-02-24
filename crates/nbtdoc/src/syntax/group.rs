#[derive(Copy, Clone, Debug, PartialEq, Eq, Hash)]
pub enum NdGroupType {
    File,
    Error,
    Item,

    IdentPath,
    MinecraftIdent,

    RegistryIndex,
    FieldPath,

    Range,

    ScalarType,
    ArrayType,
    ListType,
    IdType,
    UnionType,
    NamedType,
    IndexType,

    CompoundDef,
    CompoundExtends,
    CompoundField,

    EnumDef,
    EnumEntry,

    ModDecl,
    UseStatement,
    DescribesStatement,

    CompoundInject,
    EnumInject,

    DocCommentGroup,

    DescribesBody,
}
