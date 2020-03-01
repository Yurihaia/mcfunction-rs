// I might need some of these in the future when I decide
// to write parser unit tests
#![allow(dead_code)]

use super::{group::NdGroupType, lexer, tokens::NdTokenKind, NbtdocLang, NdParser};
use mcfunction_parse::{ast::AstView, parser::Parser, Ast, SyntaxKind, Token};

use std::fmt::Write;

pub fn format_astnode(node: AstView<&str, NbtdocLang>, indlevel: usize) -> String {
    let ind = "    ".repeat(indlevel);
    let mut out = String::new();
    match node.kind() {
        SyntaxKind::Token(tk) => match tk {
            NdTokenKind::Whitespace => write!(
                out,
                "{}Token({:?}) `{}` at {}\n",
                ind,
                tk,
                node.string().escape_debug(),
                node.span()
            )
            .unwrap(),
            NdTokenKind::DocComment | NdTokenKind::Comment => write!(
                out,
                "{}Token({:?}) `{}` at {}\n",
                ind,
                tk,
                node.string().trim(),
                node.span()
            )
            .unwrap(),
            _ => write!(
                out,
                "{}Token({:?}) `{}` at {}\n",
                ind,
                tk,
                node.string(),
                node.span()
            )
            .unwrap(),
        },
        SyntaxKind::Group(gt) => {
            write!(out, "{}Group({:?}) at {} {{\n", ind, gt, node.span()).unwrap();
            for x in node.children() {
                write!(out, "{}", format_astnode(x, indlevel + 1)).unwrap();
            }
            write!(out, "{}}}\n", ind).unwrap();
        }
        SyntaxKind::Joined(gt) => {
            write!(
                out,
                "{}Joined({:?}) `{}` at {}\n",
                ind,
                gt,
                node.string(),
                node.span()
            )
            .unwrap();
            for x in node.children() {
                if let SyntaxKind::Error(err) = x.kind() {
                    write!(out, "{}- Error `{}`\n", ind, err).unwrap()
                }
            }
        }
        SyntaxKind::Error(err) => {
            write!(out, "{}Error `{}` at {}\n", ind, err, node.span()).unwrap();
        }
        SyntaxKind::Root(kind) => {
            for x in node.children() {
                write!(out, "Root({:?})\n{}", kind, format_astnode(x, indlevel)).unwrap();
            }
        }
    };
    out
}

pub fn format_sk_list(tokens: Vec<Token<NdTokenKind>>, src: &str) -> String {
    let mut out = String::new();
    for tk in tokens {
        match tk.kind() {
            NdTokenKind::Whitespace => write!(
                out,
                "{:?} `{}` at {}\n",
                tk.kind(),
                tk.string(src).escape_debug(),
                tk.span(),
            ),
            NdTokenKind::DocComment | NdTokenKind::Comment => write!(
                out,
                "{:?} `{}` at {}\n",
                tk.kind(),
                tk.string(src).trim(),
                tk.span()
            ),
            _ => write!(
                out,
                "{:?} `{}` at {}\n",
                tk.kind(),
                tk.string(src),
                tk.span(),
            ),
        }
        .unwrap();
    }
    out
}

pub fn parse<'a, F: FnMut(&mut NdParser)>(i: &'a str, mut f: F) -> Ast<&'a str, NbtdocLang> {
    let tokens = lexer::tokenize_str(i);
    assert!(!tokens.is_empty(), "Token stream is empty");
    let mut parser = Parser::new(&tokens, i, NdGroupType::File, true);
    f(&mut parser);
    parser.build(false)
}
