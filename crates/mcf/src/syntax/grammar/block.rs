use super::*;
use crate::syntax::{group::McGroupType::*, McParser, McTokenKind::*};
use mcfunction_parse::parser::StartInfo::{self, Skip};

pub fn state(p: &mut McParser) {
    let mk = p.start(BlockState, StartInfo::None);
    resource_location(p);
    if p.at(LBracket) {
        let argmk = p.start(BlockStateArguments, Skip);
        if !p.at(RBracket) {
            loop {
                uq_string(p);
                p.expect(Eq);
                uq_string(p);
                if p.at(RBracket) {
                    break;
                }
                p.expect(Comma);
            }
        }
        p.expect(RBracket);
        p.finish(argmk);
    }
    if p.at(LCurly) {
        nbt::compound(p);
    }
    p.finish(mk);
}

pub fn predicate(p: &mut McParser) {
    let mk = p.start(BlockState, StartInfo::None);
    p.eat(Hash);
    resource_location(p);
    if p.at(LBracket) {
        let argmk = p.start(BlockStateArguments, Skip);
        if !p.at(RBracket) {
            loop {
                uq_string(p);
                p.expect(Eq);
                uq_string(p);
                if p.at(RBracket) {
                    break;
                }
                p.expect(Comma);
            }
        }
        p.expect(RBracket);
        p.finish(argmk);
    }
    if p.at(LCurly) {
        nbt::compound(p);
    }
    p.finish(mk);
}
