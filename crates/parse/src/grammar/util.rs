use crate::{
    ast::GroupType::{self, *},
    parser::{Parser, StartInfo, TokenParser},
    syntax::TokenKind::*,
    tokenset, TokenSet,
};

pub fn resource_location(p: &mut Parser) {
    if !p.try_token(resource_location_tk, ResourceLocation) {
        p.error(ResourceLocation);
    }
}

pub fn resource_location_tk(p: &mut TokenParser) -> Option<()> {
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

pub fn range(p: &mut Parser) {
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

pub fn try_range(p: &mut Parser) -> bool {
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

pub const ALLOWED_UQ_STRING: TokenSet = tokenset![Digits, Word, Dash, Plus, Dot, DotDot];

pub const BOOLEAN: &[(&str, GroupType)] = &[("true", BooleanTrue), ("false", BooleanFalse)];

pub const FLOAT_SCI: &[(&str, GroupType)] = &[("e", FloatSciExpLower), ("E", FloatSciExpUpper)];

pub fn uq_string(p: &mut Parser) {
    p.try_token(uq_string_tk, UnquotedString);
}

pub fn uq_string_tk(p: &mut TokenParser) -> Option<()> {
    while p.eat_tokens(ALLOWED_UQ_STRING) {}
    Some(())
}

pub fn uq_string_ne_tk(p: &mut TokenParser) -> Option<()> {
    p.expect_tokens(ALLOWED_UQ_STRING)?;
    uq_string_tk(p)?;
    Some(())
}

pub fn string(p: &mut Parser) {
    if !p.eat(QuotedString) {
        p.try_token(uq_string_tk, UnquotedString);
    }
}

pub fn integer(p: &mut Parser) {
    if !p.try_token(integer_tk, Integer) {
        p.error(Integer);
    }
}

pub fn integer_tk(p: &mut TokenParser) -> Option<()> {
    if !p.eat(Dash) {
        p.eat(Plus);
    }
    p.expect(Digits)?;
    Some(())
}

pub fn float(p: &mut Parser) {
    if !p.try_token(float_tk, Float) {
        p.error(Float);
    }
}

pub fn float_tk(p: &mut TokenParser) -> Option<()> {
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
    use crate::{parse, parser::Parser, testing::format_astnode};
    use insta::assert_snapshot;

    macro_rules! util_test {
        ($name:ident, $e:expr, $f:expr) => {
            #[test]
            fn $name() {
                assert_snapshot!(test($e, $f));
            }
        };
    }

    fn test<F: FnOnce(&mut Parser)>(i: &str, f: F) -> String {
        format_astnode(&parse(i, f), 0)
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
