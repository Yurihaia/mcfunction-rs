use crate::{
    ast::{AstNode, SyntaxKind},
    mcf::{McGroupType, McTokenKind, McfLang},
    Token,
};

use std::fmt::Write;

pub fn format_astnode(node: &AstNode<McfLang>, indlevel: usize) -> String {
    let ind = "    ".repeat(indlevel);
    let mut out = String::new();
    match node.kind() {
        SyntaxKind::Token(tk) => {
            if let McTokenKind::Whitespace | McTokenKind::LineBreak = tk {
                write!(
                    out,
                    "{}Token({:?}) `{}` at {}\n",
                    ind,
                    tk,
                    node.string().escape_debug(),
                    node.span()
                )
                .unwrap();
            } else {
                write!(
                    out,
                    "{}Token({:?}) `{}` at {}\n",
                    ind,
                    tk,
                    node.string(),
                    node.span()
                )
                .unwrap();
            }
        }
        SyntaxKind::Group(gt) => {
            write!(
                out,
                "{}Group({}) at {} {{\n",
                ind,
                match gt {
                    McGroupType::CommandNode(ind) => format!("CommandNode({})", usize::from(*ind)),
                    v => format!("{:?}", v),
                },
                node.span()
            )
            .unwrap();
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
        SyntaxKind::Root => {
            for x in node.children() {
                write!(out, "{}", format_astnode(x, indlevel)).unwrap();
            }
        }
    };
    out
}

pub fn format_sk_list(tokens: Vec<Token<McTokenKind>>, src: &str) -> String {
    let mut out = String::new();
    for tk in tokens {
        match tk.kind() {
            McTokenKind::Whitespace | McTokenKind::LineBreak => write!(
                out,
                "{:?} `{}` at {}\n",
                tk.kind(),
                tk.string(src).escape_debug(),
                tk.span(),
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
