use super::*;
use crate::{
    mcf::{
        group::McGroupType::{self, *},
        syntax::McTokenKind::*,
        McParser,
    },
    parser::StartInfo::{self, Join, Skip},
    tokenset,
};

pub const NBT_NUMBER_SUFFIX: &[(&str, McGroupType)] = &[
    ("b", NbtSuffixB),
    ("s", NbtSuffixS),
    ("l", NbtSuffixL),
    ("f", NbtSuffixF),
    ("d", NbtSuffixD),
];

pub const NBT_SEQ_PREFIX: &[(&str, McGroupType)] =
    &[("B", NbtPrefixB), ("I", NbtPrefixI), ("L", NbtPrefixL)];

pub fn value(p: &mut McParser) {
    if p.at(QuotedString) {
        p.bump();
    } else if p.at(LCurly) {
        compound(p);
    } else if p.at(LBracket) {
        let mk = p.start(NbtSequence, Skip);

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
    } else if p.at(Word) && !tokenset!(p.nth(1) => ALLOWED_UQ_STRING) && p.at_keyword(BOOLEAN) {
        let mk = p.start(NbtBoolean, Join);
        p.bump();
        p.finish(mk);
    } else {
        let nmk = p.start(NbtNumber, StartInfo::None);

        if !p.try_token(float_tk, Float) {
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

pub fn compound(p: &mut McParser) {
    let cpdmk = p.start(NbtCompound, Skip);

    p.bump();
    if !p.at(RCurly) {
        loop {
            let enmk = p.start(NbtCompoundEntry, Skip);
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
}

pub fn path(p: &mut McParser) {
    let mk = p.start(NbtPath, StartInfo::None);
    let mut start = true;
    loop {
        let vmk = p.start(NbtPathSegment, StartInfo::None);
        if p.at(LBracket) {
            let indmk = p.start(NbtPathIndex, StartInfo::Skip);
            p.bump();
            if !p.at(RBracket) && !p.try_token(integer_tk, Integer) {
                compound(p);
            }
            p.expect(RBracket);
            p.finish(indmk);
        } else {
            if !start {
                if !(p.eat(Dot) || p.eat(DotDot)) {
                    p.cancel(vmk);
                    break;
                }
            } else {
                start = false;
            }
            if !p.eat(Word) && !p.eat(QuotedString) {
                p.error(NbtPathSegment);
                p.finish(vmk);
                break;
            }
        }
        p.finish(vmk);
    }
    p.finish(mk);
}

#[cfg(test)]
mod tests {
    use crate::mcf::{parse, testing::format_astnode};

    use insta::assert_snapshot;

    macro_rules! nbt_test {
        ($name:ident, $e:expr) => {
            #[test]
            fn $name() {
                assert_snapshot!(parse_nbt($e));
            }
        };
    }

    macro_rules! path_test {
        ($name:ident, $e:expr) => {
            #[test]
            fn $name() {
                assert_snapshot!(parse_path($e));
            }
        };
    }

    fn parse_nbt(i: &str) -> String {
        format_astnode(&parse(i, super::value), 0)
    }

    fn parse_path(i: &str) -> String {
        format_astnode(&parse(i, super::path), 0)
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

    path_test!(empty, "");
    path_test!(single_field, "foo");
    path_test!(multi_field, "path.to.field");
    path_test!(start_index, "[0]");
    path_test!(chained_index, "[0][34][12553]");
    path_test!(mixed, "foo[0].bar.baz[1]");
    path_test!(filter, "filter[{me:123}]");
}
