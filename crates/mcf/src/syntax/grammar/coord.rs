use super::*;
use crate::syntax::{
    group::McGroupType::*,
    McParser,
    McTokenKind::{self, *},
};
use mcfunction_parse::{parser::StartInfo, tokenset, TokenSet};

pub const COORD_MODIFIER: TokenSet<McTokenKind> = tokenset![Tilde, Caret];

pub fn coord2(p: &mut McParser) {
    let mk = p.start(Coord, StartInfo::None);
    let pmk = p.start(CoordPart, StartInfo::None);
    p.eat_tokens(COORD_MODIFIER);
    if !p.at(Whitespace) {
        float(p);
    }
    p.finish(pmk);
    p.expect(Whitespace);
    let pmk = p.start(CoordPart, StartInfo::None);
    p.eat_tokens(COORD_MODIFIER);
    if !p.at(Whitespace) {
        float(p);
    }
    p.finish(pmk);
    p.finish(mk);
}

pub fn coord(p: &mut McParser) {
    let mk = p.start(Coord, StartInfo::None);
    let pmk = p.start(CoordPart, StartInfo::None);
    p.eat_tokens(COORD_MODIFIER);
    if !p.at(Whitespace) {
        float(p);
    }
    p.finish(pmk);
    p.expect(Whitespace);
    let pmk = p.start(CoordPart, StartInfo::None);
    p.eat_tokens(COORD_MODIFIER);
    if !p.at(Whitespace) {
        float(p);
    }
    p.finish(pmk);
    p.expect(Whitespace);
    let pmk = p.start(CoordPart, StartInfo::None);
    p.eat_tokens(COORD_MODIFIER);
    if !p.at_tokens(tokenset![Whitespace, Eof]) {
        float(p);
    }
    p.finish(pmk);
    p.finish(mk);
}

#[cfg(test)]
mod tests {
    use crate::syntax::testing::{format_astnode, parse};

    use insta::assert_snapshot;

    macro_rules! coord_test {
        ($name:ident, $e:expr) => {
            #[test]
            fn $name() {
                assert_snapshot!(parse_coord($e));
            }
        };
    }

    fn parse_coord(i: &str) -> String {
        format_astnode(parse(i, super::coord).root(), 0)
    }

    coord_test!(all_absolute, "0 1 2");
    coord_test!(all_relative, "~0 ~5 ~1");
    coord_test!(relative_empty, "~ ~ ~7");
    coord_test!(absolute_float, "0.3 9.65 -.14927");
    coord_test!(mixed_mod, "5.3475 ^1 ~-1000000000000");
    coord_test!(two_parts, "~12 ~3");
    coord_test!(bad_whitespace, "0   \t   ^-124 ~-0124");
}
