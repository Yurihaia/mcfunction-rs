use super::{group::McGroupType, lexer, tokens::McTokenKind, McParser, McfLang};
use mcfunction_parse::{parser::Parser, Ast, AstView, SyntaxKind, Token};

use std::fmt::Write;

pub fn format_astnode(node: AstView<&str, McfLang>, indlevel: usize) -> String {
    let ind = "    ".repeat(indlevel);
    let mut out = String::new();
    match node.kind() {
        SyntaxKind::Token(tk) => {
            if let McTokenKind::Whitespace = tk {
                writeln!(
                    out,
                    "{}Token({:?}) `{}` at {}",
                    ind,
                    tk,
                    node.string().escape_debug(),
                    node.span()
                )
                .unwrap();
            } else {
                writeln!(
                    out,
                    "{}Token({:?}) `{}` at {}",
                    ind,
                    tk,
                    node.string(),
                    node.span()
                )
                .unwrap();
            }
        }
        SyntaxKind::Group(gt) => {
            writeln!(
                out,
                "{}Group({}) at {} {{",
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
            writeln!(out, "{}}}", ind).unwrap();
        }
        SyntaxKind::Joined(gt) => {
            writeln!(
                out,
                "{}Joined({:?}) `{}` at {}",
                ind,
                gt,
                node.string(),
                node.span()
            )
            .unwrap();
            for x in node.children() {
                if let SyntaxKind::Error(err) = x.kind() {
                    writeln!(out, "{}- Error `{}`", ind, err).unwrap()
                }
            }
        }
        SyntaxKind::Error(err) => {
            writeln!(out, "{}Error `{}` at {}", ind, err, node.span()).unwrap();
        }
        SyntaxKind::Root(kind) => {
            for x in node.children() {
                write!(out, "Root({:?})\n{}", kind, format_astnode(x, indlevel)).unwrap();
            }
        }
    };
    out
}

pub fn format_sk_list(tokens: Vec<Vec<Token<McTokenKind>>>, src: &str) -> String {
    let mut out = String::new();
    for line in tokens {
        for tk in line {
            match tk.kind() {
                McTokenKind::Whitespace => writeln!(
                    out,
                    "{:?} `{}` at {}",
                    tk.kind(),
                    tk.string(src).escape_debug(),
                    tk.span(),
                ),
                McTokenKind::Eof if tk.start() != tk.end() => writeln!(
                    out,
                    "LineBreak `{}` at {}",
                    tk.string(src).escape_debug(),
                    tk.span(),
                ),
                _ => writeln!(out, "{:?} `{}` at {}", tk.kind(), tk.string(src), tk.span(),),
            }
            .unwrap();
        }
    }
    out
}

pub fn parse<'a, F: FnMut(&mut McParser)>(i: &'a str, mut f: F) -> Ast<&'a str, McfLang> {
    let tokens = lexer::tokenize_str(i);
    assert!(!tokens.is_empty(), "Token stream is empty");
    let mut parser = Parser::new(&tokens[0], i, McGroupType::File, false);
    f(&mut parser);
    for line in &tokens[1..] {
        parser.change_tokens(&line);
        f(&mut parser);
    }
    parser.build(false)
}
