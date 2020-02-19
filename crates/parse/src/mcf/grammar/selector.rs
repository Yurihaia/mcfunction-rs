use super::*;
use crate::{
    mcf::{
        group::McGroupType::{self, *},
        syntax::McTokenKind::*,
        McParser,
    },
    parser::StartInfo::{self, Skip},
    TokenSet,
};

const SELECTOR_TYPE: &[(&str, McGroupType)] = &[
    ("p", SelectorModP),
    ("a", SelectorModA),
    ("r", SelectorModR),
    ("s", SelectorModS),
    ("e", SelectorModE),
];

pub fn game_profile(p: &mut McParser) {
    let mk = p.start(Selector, StartInfo::None);
    if !p.try_token(uuid_tk, Uuid) {
        uq_string(p);
    }
    p.finish(mk);
}

pub fn entity(p: &mut McParser) {
    let mk = p.start(Selector, StartInfo::None);
    if p.at(At) {
        selector(p);
    } else if !p.try_token(uuid_tk, Uuid) {
        uq_string(p);
    }
    p.finish(mk);
}

pub fn score_holder(p: &mut McParser) {
    let mk = p.start(Selector, StartInfo::None);
    if p.at(At) {
        selector(p);
    } else if !p.try_token(uuid_tk, Uuid) {
        let nmp = p.start(UnquotedString, StartInfo::Join);
        while !p.at(Whitespace) {
            p.bump();
        }
        p.finish(nmp);
    }
    p.finish(mk);
}

pub fn selector(p: &mut McParser) {
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
}

pub fn seletor_arg_value(p: &mut McParser) {
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
        if !try_range_suffix(p) {
            resource_location(p);
        }
    }
}

pub fn try_range_suffix(p: &mut McParser) -> bool {
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
    if p.at_tokens(ALLOWED_UQ_STRING) {
        p.cancel(mk);
        return false;
    }
    p.finish(mk);
    true
}

#[cfg(test)]
mod tests {
    use crate::mcf::testing::{format_astnode, parse};

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
        format_astnode(&parse(i, super::entity), 0)
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
