use crate::{
    mcf::{
        group::McGroupType::{self, *},
        syntax::McTokenKind::{self, *},
        McParser, McfLang,
    },
    parser::{StartInfo, TokenParser},
    tokenset, TokenSet,
};

type McTokenParser<'a, 'b, 'c> = TokenParser<'a, 'b, 'c, McfLang>;

pub fn function(p: &mut McParser) {
    let mk = p.start(Function, StartInfo::None);
    p.eat(Hash);
    resource_location(p);
    p.finish(mk);
}

pub fn item_stack(p: &mut McParser) {
    let mk = p.start(ItemStack, StartInfo::None);
    resource_location(p);
    if p.at(LCurly) {
        super::nbt::compound(p);
    }
    p.finish(mk);
}

pub fn item_predicate(p: &mut McParser) {
    let mk = p.start(ItemPredicate, StartInfo::None);
    p.eat(Hash);
    resource_location(p);
    if p.at(LCurly) {
        super::nbt::compound(p);
    }
    p.finish(mk);
}

pub fn message(p: &mut McParser) {
    let mk = p.start(UnquotedString, StartInfo::None);
    while !(p.at(Eof)) {
        p.bump();
    }
    p.finish(mk);
}

pub fn resource_location(p: &mut McParser) {
    if !p.try_token(resource_location_tk, ResourceLocation) {
        p.error(ResourceLocation);
    }
}

pub fn resource_location_tk(p: &mut McTokenParser) -> Option<()> {
    uq_string_tk(p);
    if p.eat_tokens(tokenset![Colon, Slash]) {
        loop {
            uq_string_tk(p);
            if !p.eat(Slash) {
                break;
            }
        }
    }
    Some(())
}

pub fn range(p: &mut McParser) {
    let mk = p.start(Range, StartInfo::None);
    if p.eat(DotDot) {
        float(p);
    } else {
        float(p);
        if p.eat(DotDot) {
            p.try_token(float_tk, Float);
        }
    }
    p.finish(mk);
}

pub fn try_range(p: &mut McParser) -> bool {
    let mk = p.start(Range, StartInfo::None);
    if p.eat(DotDot) {
        if !p.try_token(float_tk, Float) {
            p.cancel(mk);
            return false;
        }
    } else {
        if !p.try_token(float_tk, Float) {
            p.cancel(mk);
            return false;
        }
        if p.eat(DotDot) {
            p.try_token(float_tk, Float);
        }
    }
    p.finish(mk);
    true
}

pub const ALLOWED_UQ_STRING: TokenSet<McTokenKind> =
    tokenset![Digits, Word, Dash, Plus, Dot, DotDot];

pub const OPERATION: TokenSet<McTokenKind> =
    tokenset![AddAssign, SubAssign, MulAssign, DivAssign, ModAssign, Lte, Gte, Swap];

pub const BOOLEAN: &[(&str, McGroupType)] = &[("true", BooleanTrue), ("false", BooleanFalse)];

pub const FLOAT_SCI: &[(&str, McGroupType)] = &[("e", FloatSciExpLower), ("E", FloatSciExpUpper)];

const HEX_CHAR: &[(&str, McGroupType)] = &[
    ("a", Integer),
    ("b", Integer),
    ("c", Integer),
    ("d", Integer),
    ("e", Integer),
    ("f", Integer),
];

pub const TIME_SUFFIX: &[(&str, McGroupType)] = &[("s", TimeS), ("t", TimeT), ("d", TimeD)];

pub fn uq_string(p: &mut McParser) {
    p.try_token(uq_string_tk, UnquotedString);
}

pub fn uq_string_tk(p: &mut McTokenParser) -> Option<()> {
    while p.eat_tokens(ALLOWED_UQ_STRING) {}
    Some(())
}

pub fn uq_string_ne_tk(p: &mut McTokenParser) -> Option<()> {
    p.expect_tokens(ALLOWED_UQ_STRING)?;
    uq_string_tk(p)?;
    Some(())
}

pub fn time(p: &mut McParser) {
    let mk = p.start(Time, StartInfo::None);
    p.expect(Digits);
    p.eat_keyword(TIME_SUFFIX);
    p.finish(mk);
}

pub fn uuid_tk(p: &mut McTokenParser) -> Option<()> {
    let mut empty = true;
    while p.eat(Digits) || p.eat_kw(HEX_CHAR) {
        empty = false
    }
    if empty {
        return None;
    }
    p.expect(Dash)?;
    let mut empty = true;
    while p.eat(Digits) || p.eat_kw(HEX_CHAR) {
        empty = false
    }
    if empty {
        return None;
    }
    p.expect(Dash)?;
    let mut empty = true;
    while p.eat(Digits) || p.eat_kw(HEX_CHAR) {
        empty = false
    }
    if empty {
        return None;
    }
    p.expect(Dash)?;
    let mut empty = true;
    while p.eat(Digits) || p.eat_kw(HEX_CHAR) {
        empty = false
    }
    if empty {
        return None;
    }
    Some(())
}

pub fn string(p: &mut McParser) {
    if !p.eat(QuotedString) {
        p.try_token(uq_string_tk, UnquotedString);
    }
}

pub fn integer(p: &mut McParser) {
    if !p.try_token(integer_tk, Integer) {
        p.error(Integer);
    }
}

pub fn integer_tk(p: &mut McTokenParser) -> Option<()> {
    if !p.eat(Dash) {
        p.eat(Plus);
    }
    p.expect(Digits)?;
    Some(())
}

pub fn float(p: &mut McParser) {
    if !p.try_token(float_tk, Float) {
        p.error(Float);
    }
}

pub fn float_tk(p: &mut McTokenParser) -> Option<()> {
    p.eat_tokens(tokenset![Plus, Dash]);
    if p.eat(Dot) {
        p.expect(Digits)?;
    } else if p.eat(Digits) {
        if p.eat(Dot) {
            p.eat(Digits);
        }
    } else {
        return None;
    }
    if p.eat_kw(FLOAT_SCI) {
        integer_tk(p)?;
    }
    Some(())
}

#[cfg(test)]
mod tests {
    use super::*;
    use crate::mcf::{
        testing::{format_astnode, parse},
        McParser,
    };
    use insta::assert_snapshot;

    macro_rules! util_test {
        ($name:ident, $e:expr, $f:expr) => {
            #[test]
            fn $name() {
                assert_snapshot!(test($e, $f));
            }
        };
    }

    fn test<F: FnMut(&mut McParser)>(i: &str, f: F) -> String {
        format_astnode(parse(i, f).root(), 0)
    }

    util_test!(uq_string_single_word, "hello_world", uq_string);
    util_test!(uq_string_numlike, "-1233.86+534-", uq_string);
    util_test!(uq_string_mixed, "123qvr-wvg35.+", uq_string);
    util_test!(uq_string_leftover, "hello_word; rest of input", uq_string);

    util_test!(unsigned_int, "642345", integer);
    util_test!(signed_int, "-23445", integer);
    util_test!(int_only_dash, "-", integer);
    util_test!(plus_int, "+5356", integer);

    util_test!(resloc_single, "my_location", resource_location);
    util_test!(resloc_multi, "path/to/somethig", resource_location);
    util_test!(resloc_ns_single, "namespace:single", resource_location);
    util_test!(
        resloc_ns_multi,
        "namespace:path/to/thing",
        resource_location
    );
    util_test!(resloc_ns_empty, ":", resource_location);
    util_test!(resloc_empty, "", resource_location);
}
