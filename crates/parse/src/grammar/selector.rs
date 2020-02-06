use super::*;
use crate::{
    ast::GroupType::{self, *},
    parser::{
        Parser,
        StartInfo::{self, Skip},
    },
    syntax::TokenKind::*,
    TokenSet,
};

const SELECTOR_TYPE: &[(&str, GroupType)] = &[
    ("p", SelectorModP),
    ("a", SelectorModA),
    ("r", SelectorModR),
    ("s", SelectorModS),
    ("e", SelectorModE),
];

pub fn selector(p: &mut Parser) {
    let mk = p.start(Selector, StartInfo::None);
    p.expect(At);
    if !p.expect_keyword(SELECTOR_TYPE) {
        p.bump_recover(TokenSet::empty());
    }
    if p.at(LBracket) {
        let argsmk = p.start(SelectorArgument, Skip);
        p.bump();
        if !p.at(RBracket) {
            loop {
                let argmk = p.start(SelectorArgumentEntry, Skip);
                uq_string(p);
                p.expect(Eq);
                seletor_arg_value(p);
                p.finish(argmk);
                if p.at(RBracket) {
                    break;
                }
                p.expect(Comma);
                if p.at(Eof) {
                    break;
                }
            }
        }
        p.expect(RBracket);
        p.finish(argsmk);
    }
    p.finish(mk);
}

pub fn seletor_arg_value(p: &mut Parser) {
    p.eat(Excl);
    if p.at(QuotedString) {
        p.bump();
    } else if p.at(LCurly) {
        let mapmk = p.start(SelectorArgumentMap, Skip);
        p.bump();
        if !p.at(RCurly) {
            loop {
                let argmk = p.start(SelectorArgumentMapEntry, Skip);
                resource_location(p);
                p.expect(Eq);
                seletor_arg_value(p);
                p.finish(argmk);
                if p.at(RCurly) {
                    break;
                }
                p.expect(Comma);
                if p.at(Eof) {
                    break;
                }
            }
        }
        p.expect(RCurly);
        p.finish(mapmk);
    } else {
        if !try_range(p) {
            resource_location(p);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::{parse, testing::format_astnode};

    use insta::assert_snapshot;

    macro_rules! selector_test {
        ($name:ident, $e:expr) => {
            #[test]
            fn $name() {
                assert_snapshot!(parse_selector($e));
            }
        };
    }

    fn parse_selector(i: &str) -> String {
        format_astnode(&parse(i, super::selector), 0)
    }

    selector_test!(no_args, "@p");
    selector_test!(empty_args, "@e[]");
    selector_test!(args_word, "@s[tag=hello]");
    selector_test!(args_range, "@r[distance=1..17.5]");
    selector_test!(args_map_empty, "@a[scores={}]");
    selector_test!(
        args_map_rid,
        "@p[advancements={path/to/adv={criteria=false}}]"
    );
    selector_test!(args_map_range, "@e[scores={myobjective=-12..74}]");
    selector_test!(args_multiple, "@s[tag=hello,tag=goodbye,scores={}]");
    selector_test!(arg_invert, "@r[type=!minecraft:pig]");

    selector_test!(invalid_mod, "@q[name=\"hello\"]");
    selector_test!(unclosed_arg_empty, "@e[");
    selector_test!(unclosed_arg_key, "@p[type");
    selector_test!(arg_unclosed_map_nokey, "@s[score={");
    selector_test!(arg_unclosed_map_noeq, "@p[advancements={hello");
}
