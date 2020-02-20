use util::commands::Index;

#[derive(Debug, Copy, Clone, PartialEq, Eq)]
pub enum McGroupType {
    File,
    CommandNode(Index),
    Command,
    // NBT groups
    NbtCompound,
    NbtSequence,
    NbtNumber,
    NbtString,
    NbtBoolean,
    // NBT Keywords
    NbtSuffixB,
    NbtSuffixS,
    NbtSuffixL,
    NbtSuffixF,
    NbtSuffixD,
    NbtPrefixB,
    NbtPrefixI,
    NbtPrefixL,
    // NBT Compound
    NbtCompoundEntry,

    NbtPath,
    NbtPathSegment,
    NbtPathIndex,

    // Selector groups
    Selector,
    SelectorArgument,
    SelectorArgumentEntry,
    SelectorArgumentMap,
    SelectorArgumentMapEntry,
    // Selector keywords
    SelectorModP,
    SelectorModA,
    SelectorModR,
    SelectorModS,
    SelectorModE,

    // Block states
    BlockState,
    BlockStateArguments,

    ItemStack,
    ItemPredicate,

    Comment,

    Function,

    JsonObject,
    JsonObjectEntry,
    JsonList,
    JsonNull,

    Integer,
    Float,
    UnquotedString,
    ResourceLocation,
    Range,
    Uuid,
    Time,

    TimeS,
    TimeT,
    TimeD,

    Coord,
    CoordPart,

    // Keywords
    BooleanTrue,
    BooleanFalse,

    FloatSciExpUpper,
    FloatSciExpLower,

    // Special error type that signifies an error
    Error,
}
