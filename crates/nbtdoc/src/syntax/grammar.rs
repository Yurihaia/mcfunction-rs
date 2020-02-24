use mcfunction_parse::{
    parser::StartInfo::{self, Skip},
    tokenset, TokenKind, TokenSet,
};

use super::{
    group::NdGroupType::*,
    tokens::NdTokenKind::{self, *},
    NdParser as Parser,
};

pub fn file(p: &mut Parser) {
    let mk = p.start(File, Skip);
    loop {
        let mk = p.start(Item, Skip);
        doc_comments(p);
        let mut lk = p.lookahead();
        if lk.at(CompoundKw) {
            compound(p);
        } else if lk.at(EnumKw) {
            enum_def(p);
        } else if lk.at(ModKw) {
            let mmk = p.start(ModDecl, Skip);
            p.bump();
            p.expect(Ident);
            p.expect(Semicolon);
            p.finish(mmk);
        } else if lk.at(UseKw) || lk.at(ExportKw) {
            let umk = p.start(UseStatement, Skip);
            if !p.eat(UseKw) {
                p.bump();
                p.expect(UseKw);
            }
            ident_path(p);
            p.expect(Semicolon);
            p.finish(umk);
        } else if lk.at(InjectKw) {
            inject(p);
        } else if lk.at(Ident) || lk.at(ColonColon) {
            let mk = p.start(DescribesStatement, Skip);
            ident_path(p);
            p.expect(DescribesKw);
            minecraft_ident(p);
            if p.eat(LBracket) {
                let bmk = p.start(DescribesBody, Skip);
                while p.not_at(RBracket) {
                    minecraft_ident(p);
                    if !p.at(RBracket) {
                        p.expect(Comma);
                    }
                }
                p.finish(bmk);
                p.expect(RBracket);
            }
            p.expect(Semicolon);
            p.finish(mk);
        } else {
            lk.add_errors();
            p.bump();
        }
        p.finish(mk);
        if p.at(Eof) {
            break;
        }
    }
    p.finish(mk);
}

pub fn compound(p: &mut Parser) {
    let cpmk = p.start(CompoundDef, Skip);
    p.expect(CompoundKw);
    p.expect(Ident);
    if p.at(ExtendsKw) {
        let exmk = p.start(CompoundExtends, Skip);
        p.bump();
        if index_over_ident(p) {
            registry_index(p);
        } else {
            ident_path(p);
        }
        p.finish(exmk);
    }
    p.expect(LCurly);
    while p.not_at(RCurly) {
        let fmk = p.start(CompoundField, Skip);
        doc_comments(p);
        ident_or_qs(p);
        p.expect(Colon);
        field_type(p);
        p.finish(fmk);
        if !p.at(RCurly) {
            p.expect(Comma);
        }
    }
    p.expect(RCurly);
    p.finish(cpmk);
}

pub fn enum_def(p: &mut Parser) {
    let enmk = p.start(EnumDef, Skip);
    p.expect(EnumKw);
    p.expect(LParen);
    let mut lk = p.lookahead();
    if lk.at(ByteKw)
        || lk.at(ShortKw)
        || lk.at(IntKw)
        || lk.at(LongKw)
        || lk.at(FloatKw)
        || lk.at(DoubleKw)
        || lk.at(StringKw)
        || lk.at(BooleanKw)
    {
        p.bump();
    } else {
        lk.add_errors();
        p.bump_recover(TokenSet::empty());
    }
    p.expect(RParen);
    p.expect(Ident);
    p.expect(LCurly);
    while p.not_at(RCurly) {
        let mk = p.start(EnumEntry, Skip);
        doc_comments(p);
        p.expect(Ident);
        p.expect(Eq);
        let mut lk = p.lookahead();
        if lk.at(QuotedString) || lk.at(Float) {
            p.bump();
        } else {
            lk.add_errors();
        }
        p.finish(mk);
        if !p.at(RCurly) {
            p.expect(Comma);
        }
    }
    p.expect(RCurly);
    p.finish(enmk);
}

pub fn inject(p: &mut Parser) {
    let mk = p.start(Error, Skip);
    p.expect(InjectKw);
    let mut lk = p.lookahead();
    if lk.at(CompoundKw) {
        p.retype(&mk, CompoundInject, false);
        p.bump();
        ident_path(p);
        p.expect(LCurly);
        while p.not_at(RCurly) {
            let fmk = p.start(CompoundField, Skip);
            doc_comments(p);
            ident_or_qs(p);
            p.expect(Colon);
            field_type(p);
            p.finish(fmk);
            if !p.at(RCurly) {
                p.expect(Comma);
            }
        }
        p.expect(RCurly);
    } else if lk.at(EnumKw) {
        p.retype(&mk, EnumInject, false);
        p.bump();
        let mut lk = p.lookahead();
        if lk.at(ByteKw)
            || lk.at(ShortKw)
            || lk.at(IntKw)
            || lk.at(LongKw)
            || lk.at(FloatKw)
            || lk.at(DoubleKw)
            || lk.at(StringKw)
            || lk.at(BooleanKw)
        {
            p.bump();
        } else {
            lk.add_errors();
            p.bump_recover(TokenSet::empty());
        }
        ident_path(p);
        p.expect(LCurly);
        while p.not_at(RCurly) {
            let mk = p.start(EnumEntry, Skip);
            doc_comments(p);
            p.expect(Ident);
            p.expect(Eq);
            let mut lk = p.lookahead();
            if lk.at(QuotedString) || lk.at(Float) {
                p.bump();
            } else {
                lk.add_errors();
            }
            p.finish(mk);
            if !p.at(RCurly) {
                p.expect(Comma);
            }
        }
        p.expect(RCurly);
    } else {
        lk.add_errors();
    }
    p.finish(mk);
}

pub fn ident_path(p: &mut Parser) {
    let mk = p.start(IdentPath, StartInfo::None);
    p.eat(ColonColon);
    loop {
        let mut lk = p.lookahead();
        if lk.at(Ident) || lk.at(SuperKw) {
            p.bump();
        } else {
            lk.add_errors();
        }
        if !p.eat(ColonColon) {
            break;
        }
    }
    p.finish(mk);
}

pub fn ident_or_qs(p: &mut Parser) {
    let mut lk = p.lookahead();
    if lk.at(Ident) || lk.at(QuotedString) {
        p.bump();
    } else {
        lk.add_errors();
        p.bump_recover(tokenset![Colon]);
    }
}

pub fn minecraft_ident(p: &mut Parser) {
    let mk = p.start(MinecraftIdent, StartInfo::Join);
    if !p.eat(QuotedString) {
        if !p.at(Colon) {
            ident_or_qs(p);
        }
        p.expect(Colon);
        while p.at(Ident) || p.nth(0).is_keyword() || p.at(Slash) {
            p.bump();
        }
    }
    p.finish(mk);
}

pub fn registry_index(p: &mut Parser) {
    let mk = p.start(RegistryIndex, Skip);
    minecraft_ident(p);
    p.expect(LBracket);
    let fmk = p.start(FieldPath, StartInfo::None);
    while p.not_at(RBracket) {
        let mut lk = p.lookahead();
        if lk.at(Ident) || lk.at(QuotedString) || lk.at(SuperKw) {
            p.bump();
        } else {
            lk.add_errors();
            p.bump_recover(tokenset![Dot]);
        }
        if !p.at(RBracket) {
            p.expect(Dot);
        }
    }
    p.finish(fmk);
    p.expect(RBracket);
    p.finish(mk);
}

pub fn field_type(p: &mut Parser) {
    let mk = p.start(Error, Skip);
    let mut lk = p.lookahead();
    if lk.at(ByteKw)
        || lk.at(ShortKw)
        || lk.at(IntKw)
        || lk.at(LongKw)
        || lk.at(FloatKw)
        || lk.at(DoubleKw)
        || lk.at(StringKw)
        || lk.at(BooleanKw)
    {
        p.bump();
        if p.at(At) {
            range(p);
        }
        if p.at(LBracket) {
            p.bump();
            p.expect(RBracket);
            if p.at(At) {
                range(p);
            }
            p.retype(&mk, ArrayType, false);
        } else {
            p.retype(&mk, ScalarType, false);
        }
    } else if lk.at(LBracket) {
        p.retype(&mk, ListType, false);
        p.bump();
        field_type(p);
        p.expect(RBracket);
    } else if lk.at(IdKw) {
        p.retype(&mk, IdType, false);
        p.bump();
        p.expect(LParen);
        minecraft_ident(p);
        p.expect(RParen);
    } else if lk.at(LParen) {
        p.retype(&mk, UnionType, false);
        p.bump();
        while p.not_at(RParen) {
            field_type(p);
            if !p.at(RParen) {
                p.expect(Bar);
            }
        }
        p.expect(RParen);
    } else {
        let errs = lk.get_errors();
        if index_over_ident(p) {
            p.retype(&mk, IndexType, false);
            registry_index(p);
        } else if p.at_tokens(tokenset![ColonColon, Ident, SuperKw]) {
            p.retype(&mk, NamedType, false);
            ident_path(p);
        } else {
            p.add_errors(errs);
            p.error(RegistryIndex);
            p.error(IdentPath);
        }
    }
    p.finish(mk);
}

pub fn range(p: &mut Parser) {
    let mk = p.start(Range, Skip);
    p.expect(At);
    if p.eat(DotDot) {
        p.expect(Float);
    } else {
        p.expect(Float);
        if p.eat(DotDot) {
            p.eat(Float);
        }
    }
    p.finish(mk);
}

pub fn doc_comments(p: &mut Parser) {
    let mk = p.start(DocCommentGroup, StartInfo::None);
    while p.eat_tokens(NdTokenKind::WHITESPACE) {}
    p.finish(mk);
}

fn index_over_ident(p: &Parser) -> bool {
    if p.at(Colon) || p.nth(1) == Colon {
        return true;
    } else if p.at(QuotedString) && p.nth(1) == LBracket {
        return true;
    } else {
        return false;
    }
}
