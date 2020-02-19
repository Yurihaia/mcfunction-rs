use super::*;
use crate::{
    mcf::{
        group::McGroupType::{self, *},
        syntax::McTokenKind::*,
        McParser,
    },
    parser::StartInfo::*,
    TokenSet,
};

const JSON_NULL: &[(&str, McGroupType)] = &[("null", JsonNull)];

pub fn object(p: &mut McParser) {
    let objmk = p.start(JsonObject, Skip);
    p.expect(LCurly);
    if p.eat(RCurly) {
        p.finish(objmk);
    } else {
        loop {
            let entmk = p.start(JsonObjectEntry, Skip);
            if p.expect(QuotedString) && p.expect(Colon) {
                value(p);
            }
            p.finish(entmk);
            if p.at(RCurly) {
                break;
            }
            match (p.expect(Comma), p.at(Eof)) {
                (true, true) => {
                    p.expect(QuotedString);
                    break;
                }
                (false, false) => {
                    if p.bump_recover(TokenSet::empty()) {
                        break;
                    }
                }
                (true, false) => (),
                (false, true) => {
                    break;
                }
            }
        }
        p.expect(RCurly);
        p.finish(objmk);
    }
}

pub fn array(p: &mut McParser) {
    let arrmk = p.start(JsonList, Skip);
    p.expect(LBracket);
    if p.eat(RBracket) {
        p.finish(arrmk);
    } else {
        loop {
            value(p);
            if p.at(RBracket) {
                break;
            }
            match (p.expect(Comma), p.at(Eof)) {
                (true, true) => {
                    value(p);
                    break;
                }
                (false, false) => {
                    if p.bump_recover(TokenSet::empty()) {
                        break;
                    }
                }
                (true, false) => (),
                (false, true) => {
                    break;
                }
            }
        }
        p.expect(RBracket);
        p.finish(arrmk);
    }
}

pub fn value(p: &mut McParser) {
    let mut lk = p.lookahead();
    if lk.at(LCurly) {
        object(p);
    } else if lk.at(LBracket) {
        array(p);
    } else if lk.at(QuotedString) {
        p.bump();
    } else if lk.at_keywords(BOOLEAN) {
        p.eat_keyword(BOOLEAN);
    } else if lk.at_keywords(JSON_NULL) {
        p.eat_keyword(JSON_NULL);
    } else {
        lk.group_error(Float);
        let errs = lk.get_errors();
        if !p.try_token(float_tk, Float) {
            p.add_errors(errs);
        }
    }
}

#[cfg(test)]
mod tests {
    use crate::mcf::testing::{format_astnode, parse};

    use insta::assert_snapshot;

    macro_rules! json_test {
        ($name:ident, $e:expr) => {
            #[test]
            fn $name() {
                assert_snapshot!(parse_json($e));
            }
        };
    }

    fn parse_json(i: &str) -> String {
        format_astnode(&parse(i, super::value), 0)
    }

    json_test!(boolean_true, "true");
    json_test!(boolean_false, "false");
    json_test!(null, "null");
    json_test!(
        string,
        r#""hello world! This is a string\n with\n escapes""#
    );
    json_test!(object_empty, "{}");
    json_test!(object_simple, r#"{"text":"hello"}"#);
    json_test!(object_multi, r#"{"text":"hello","bold":true}"#);
    json_test!(object_nokey, r#"{"foo":true,}"#);
    json_test!(object_nocolon, r#"{"foo":true,"bar"}"#);
    json_test!(object_nokey_unclosed, r#"{"foo":true,"#);

    json_test!(array_empty, "[]");
    json_test!(array_simple, r#"[1, true, "hello"]"#);
    json_test!(array_noval, r#"[1, ]"#);
    json_test!(array_unclosed, r#"["hello", "#);
    json_test!(array_nocomma, r#"[true"#);
}
