use super::*;
use crate::{
    ast::GroupType::{self, *},
    parser::Parser,
    syntax::TokenKind::*,
};

pub const NBT_NUMBER_SUFFIX: &[(&str, GroupType)] = &[
    ("b", NbtSuffixB),
    ("s", NbtSuffixS),
    ("l", NbtSuffixL),
    ("f", NbtSuffixF),
    ("d", NbtSuffixD),
];

pub const NBT_SEQ_PREFIX: &[(&str, GroupType)] =
    &[("B", NbtPrefixB), ("I", NbtPrefixI), ("L", NbtPrefixL)];

pub fn value(p: &mut Parser) {
    if p.at(QuotedString) {
        p.bump();
    } else if p.at(LCurly) {
        let cpdmk = p.start(NbtCompound, false, true);

        p.bump();
        if !p.at(RCurly) {
            loop {
                let enmk = p.start(NbtCompoundEntry, false, true);
                string(p);
                p.expect(Colon);
                value(p);
                p.finish(enmk);
                if !p.eat(Comma) {
                    break;
                }
            }
        }
        p.expect(RCurly);
        p.finish(cpdmk);
    } else if p.at(LBracket) {
        let mk = p.start(NbtSequence, false, true);

        p.bump();
        if p.nth(1) == Semicolon {
            if !p.expect_keyword(NBT_SEQ_PREFIX) {
                p.bump();
            }
            p.bump();
        }
        if !p.at(RBracket) {
            loop {
                value(p);
                if !p.eat(Comma) {
                    break;
                }
            }
        }
        p.expect(RBracket);
        p.finish(mk);
    } else if p.at(Word) && !ALLOWED_UQ_STRING.contains(p.nth(1)) && p.at_keyword(BOOLEAN) {
        let mk = p.start(NbtBoolean, true, false);
        p.bump();
        p.finish(mk);
    } else {
        let nmk = p.start(NbtNumber, false, false);

        if !try_float(p) {
            p.cancel(nmk);
            uq_string(p);
        } else {
            p.eat_keyword(NBT_NUMBER_SUFFIX);
            if p.at_tokens(ALLOWED_UQ_STRING) {
                p.cancel(nmk);
                uq_string(p);
            } else {
                p.finish(nmk);
            }
        }
    }
}

fn try_float(p: &mut Parser) -> bool {
    let fmk = p.start(Float, true, false);

    if !p.eat(Dash) {
        p.eat(Plus);
    }
    if p.at(Dot) {
        p.bump();
        if !p.eat(Digits) {
            p.cancel(fmk);
            return false;
        }
        if p.eat_keyword(FLOAT_SCI) {
            integer(p);
        }
    } else if p.at(Digits) {
        p.bump();
        if p.eat(Dot) {
            p.eat(Digits);
            if p.eat_keyword(FLOAT_SCI) {
                integer(p);
            }
        } else {
            if p.eat_keyword(FLOAT_SCI) {
                integer(p);
            } else {
                p.retype(&fmk, Integer, true);
            }
        }
    } else {
        p.cancel(fmk);
        return false;
    }
    p.finish(fmk);
    true
}

#[cfg(test)]
mod tests {
    use crate::{parse, testing::format_astnode};

    use insta::assert_snapshot;

    macro_rules! nbt_test {
        ($name:ident, $e:expr) => {
            #[test]
            fn $name() {
                assert_snapshot!(parse_nbt($e));
            }
        };
    }

    fn parse_nbt(i: &str) -> String {
        format_astnode(&parse(i, super::value), 0)
    }

    nbt_test!(unsigned_int, "123");
    nbt_test!(signed_int, "-2147483648");
    nbt_test!(suffix_int, "16s");
    nbt_test!(invalid_suffix_int, "1a");

    nbt_test!(compound_empty, "{}");
    nbt_test!(compound_simple, "{foo:123}");
    nbt_test!(compound_multi, "{foo:123,bar:420}");
    nbt_test!(compound_spacing, "{\tfoo  :1564 ,  \t   bar:420   }");
    nbt_test!(compound_unclosed, "{foo:123,bar:420");
    nbt_test!(compound_nested, "{foo:{bar:{baz:\"ikr\"}}}");

    nbt_test!(list_empty, "[]");
    nbt_test!(array_empty, "[B;]");
    nbt_test!(list_simple, "['hello']");
    nbt_test!(array_simple, "[L;123456789]");
    nbt_test!(list_multi, "['hello', 123, true]");
    nbt_test!(list_nested, "[[123],[[],456,[789]]]");
    nbt_test!(list_unclosed, "[1, 2, 3, 4, 5, 6");

    nbt_test!(boolean_true, "true");
    nbt_test!(boolean_false, "false");

    nbt_test!(float_unsigned, "0.5772156649");
    nbt_test!(float_no_trail, "645423.");
    nbt_test!(float_sci, "2.2e10");
    nbt_test!(float_plus, "+2.718281828");
    nbt_test!(float_dash, "-0.61803398875");
    nbt_test!(float_no_prec, ".37412");
    nbt_test!(float_only_dot, ".");
}
