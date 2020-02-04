use crate::{
    ast::GroupType::{self, *},
    parser::Parser,
    syntax::TokenKind::*,
    tokenset, TokenSet,
};

pub fn resource_location(p: &mut Parser) {
    let mk = p.start(ResourceLocation, true, false);
    uq_string(p);
    if p.at(Colon) || p.eat(Slash) {
        loop {
            uq_string(p);
            if !p.eat(Slash) {
                break;
            }
        }
    }
    p.finish(mk);
}

pub fn range(p: &mut Parser) {
    let mk = p.start(Range, false, false);
    if p.eat(DotDot) {
        float(p);
    } else {
        float(p);
        if p.eat(DotDot) && p.at_tokens(ALLOWED_NUMBER_START) {
            float(p);
        }
    }
    p.finish(mk);
}

pub const ALLOWED_UQ_STRING: TokenSet = tokenset![Digits, Word, Dash, Plus, Dot];

pub const ALLOWED_NUMBER_START: TokenSet = tokenset![Digits, Dash, Plus, Dot];

pub const BOOLEAN: &[(&str, GroupType)] = &[("true", BooleanTrue), ("false", BooleanFalse)];

pub const FLOAT_SCI: &[(&str, GroupType)] = &[("e", FloatSciExpLower), ("E", FloatSciExpUpper)];

pub fn uq_string(p: &mut Parser) {
    let mk = p.start(UnquotedString, true, false);

    while p.at_tokens(ALLOWED_UQ_STRING) {
        p.bump();
    }
    p.finish(mk);
}

pub fn string(p: &mut Parser) {
    if !p.eat(QuotedString) {
        uq_string(p);
    }
}

pub fn integer(p: &mut Parser) {
    let mk = p.start(Integer, true, false);

    if !p.eat(Dash) {
        p.eat(Plus);
    }
    if !p.eat(Digits) {
        p.error(Integer);
    }
    p.finish(mk);
}

pub fn float(p: &mut Parser) {
    let mk = p.start(Float, true, false);

    if !p.eat(Dash) {
        p.eat(Plus);
    }
    if p.at(Dot) {
        p.bump();
        p.expect(Digits);
    } else if p.at(Digits) {
        p.bump();
        if p.eat(Dot) {
            p.eat(Digits);
        }
    } else {
        p.error(Float);
    }
    if p.eat_keyword(FLOAT_SCI) {
        integer(p);
    }
    p.finish(mk)
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
}
