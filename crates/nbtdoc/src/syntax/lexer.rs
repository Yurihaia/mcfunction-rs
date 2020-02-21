use super::tokens::NdTokenKind::{self, *};
use mcfunction_parse::{LineCol, Span, Token};

pub const PUNCT: &[(&str, NdTokenKind)] = &[
    ("..", DotDot),
    ("::", ColonColon),
    // Single
    (",", Comma),
    (":", Colon),
    ("@", At),
    ("=", Eq),
    ("/", Slash),
    (".", Dot),
    (";", Semicolon),
    // Delimiters
    ("{", LCurly),
    ("}", RCurly),
    ("[", LBracket),
    ("]", RBracket),
    ("(", LParen),
    (")", RParen),
];

pub const KW: &[(&str, NdTokenKind)] = &[
    ("byte", ByteKw),
    ("short", ShortKw),
    ("int", IntKw),
    ("long", LongKw),
    ("float", FloatKw),
    ("double", DoubleKw),
    ("string", StringKw),
    ("boolean", BooleanKw),
    ("mod", ModKw),
    ("compound", CompoundKw),
    ("enum", EnumKw),
    ("inject", InjectKw),
    ("super", SuperKw),
    ("extends", ExtendsKw),
    ("export", ExportKw),
    ("use", UseKw),
    ("describes", DescribesKw),
    ("id", IdKw),
];

pub fn tokenize_str<'a>(src: &'a str) -> Vec<Token<NdTokenKind>> {
    let mut off = 0;
    let mut out = vec![];
    let mut line = 0;
    let mut col = 0;
    'out: loop {
        let start = LineCol::new(line, col);
        if off >= src.len() {
            break;
        }
        if let Some(len) = try_float(src[off..].chars()) {
            col += len;
            out.push(Token::new(
                Float,
                Span::new(start, LineCol::new(line, col)),
                off,
                off + len,
            ));
            off += len;
            continue;
        }
        if src[off..].starts_with("//") {
            let doc = src[off..].starts_with("///");
            let mut len = 0;
            for c in src[off..].chars() {
                len += c.len_utf8();
                if c == '\n' {
                    break;
                }
            }
            line += 1;
            col = 0;
            out.push(Token::new(
                if doc { DocComment } else { Comment },
                Span::new(start, LineCol::new(line, col)),
                off,
                off + len,
            ));
            off += len;
        }
        for (s, t) in PUNCT {
            if src[off..].starts_with(s) {
                col += s.len();
                out.push(Token::new(
                    *t,
                    Span::new(start, LineCol::new(line, col)),
                    off,
                    off + s.len(),
                ));
                off += s.len();
                continue 'out;
            }
        }
        let cn = src[off..].chars().next().unwrap();
        if cn.is_ascii_alphabetic() || cn == '_' {
            let (end, endch) = src[off..]
                .char_indices()
                .take_while(|(_, c)| c.is_ascii_alphanumeric() || *c == '_')
                .last()
                .unwrap();
            let end = end + endch.len_utf8();
            col += end;
            let mut ty = Ident;
            for (s, t) in KW {
                if &src[off..(off + end)] == *s {
                    ty = *t;
                    break;
                }
            }
            out.push(Token::new(
                ty,
                Span::new(start, LineCol::new(line, col)),
                off,
                off + end,
            ));
            off += end;
        } else if cn == '"' || cn == '\'' {
            let mut chars = src[off..].chars();
            // consume the quotation
            chars.next().unwrap();
            let mut offset = 1; // length of quote marks is always 1 byte
            let mut escaped = false;
            while let Some(ch) = chars.next() {
                if ch == '\r' || ch == '\n' {
                    break;
                }
                offset += ch.len_utf8();
                if escaped {
                    escaped = false;
                } else if ch == cn {
                    break;
                } else if ch == '\\' {
                    escaped = true;
                }
            }
            col += offset;
            out.push(Token::new(
                QuotedString,
                Span::new(start, LineCol::new(line, col)),
                off,
                off + offset,
            ));
            off += offset;
        } else if cn.is_ascii_whitespace() {
            let mut len = 0;
            for x in src[off..].chars() {
                match x {
                    '\n' => {
                        col = 0;
                        line += 1;
                    }
                    v if v.is_ascii_whitespace() => {
                        col += 1;
                    }
                    _ => break,
                }
                len += x.len_utf8();
            }
            out.push(Token::new(
                Whitespace,
                Span::new(start, LineCol::new(line, col)),
                off,
                off + len,
            ));
            off += len;
        } else {
            let len = cn.len_utf8();
            col += len;
            out.push(Token::new(
                Invalid,
                Span::new(start, LineCol::new(line, col)),
                off,
                off + len,
            ));
            off += len;
        }
    }
    out.push(Token::new(
        NdTokenKind::Eof,
        Span::new(LineCol::new(line, col), LineCol::new(line, col)),
        off,
        off,
    ));
    out
}

fn try_float(iter: impl Iterator<Item = char> + Clone) -> Option<usize> {
    let mut iter = iter.peekable();
    let mut len = 0;
    if *iter.peek()? == '-' {
        iter.next();
        len += 1;
    }
    if iter.peek().copied() == Some('.') {
        iter.next();
        len += 1;
        if !iter.next()?.is_ascii_digit() {
            return None;
        }
        len += 1;
        while iter.peek().map(|v| v.is_ascii_digit()).unwrap_or(false) {
            iter.next();
            len += 1;
        }
    } else {
        if !iter.next()?.is_ascii_digit() {
            return None;
        }
        len += 1;
        while iter.peek().map(|v| v.is_ascii_digit()).unwrap_or(false) {
            iter.next();
            len += 1;
        }
        if iter.peek().copied() == Some('.') {
            iter.next();
            len += 1;
            while iter.peek().map(|v| v.is_ascii_digit()).unwrap_or(false) {
                iter.next();
                len += 1;
            }
        }
    }
    match iter.peek().copied() {
        Some('e') | Some('E') => {
            iter.next();
            len += 1;
            if *iter.peek()? == '-' || *iter.peek()? == '+' {
                iter.next();
                len += 1;
            }
            if !iter.next()?.is_ascii_digit() {
                return None;
            }
            len += 1;
            while iter.peek().map(|v| v.is_ascii_digit()).unwrap_or(false) {
                iter.next();
                len += 1;
            }
        }
        _ => (),
    }
    Some(len)
}

#[cfg(test)]
mod tests {
    use crate::syntax::testing::format_sk_list;
    use insta::assert_snapshot;

    macro_rules! lex_test {
        ($name:ident, $e:expr) => {
            #[test]
            fn $name() {
                assert_snapshot!(format_sk_list(super::tokenize_str($e), $e));
            }
        };
    }

    lex_test!(ident, "s0m3_1dent1f13r");
    lex_test!(keywords, "byte long int string extends compound enum");
    lex_test!(not_keyword, "bytelong hello_world");
    lex_test!(quoted_string, r#""hello world""#);

    lex_test!(punct_single, "@");
    lex_test!(misc_punct, ",::@:/..][");
    lex_test!(lit_string_escapes, r#""hello \"\q\u\o\t\e\d\\\" world""#);
    lex_test!(digits, "120394");
    lex_test!(integer, "-2147483648");
    lex_test!(float, "0.5772156649");
    lex_test!(float_exp, "314.5e-2");
    lex_test!(test_group_complex, "{.\"hello\" \n ident}");
    lex_test!(group_nested, "{ { { a } b c } d }");
    lex_test!(multi_top, "identifier 1023:7");
    lex_test!(whitespace, "   \t    \t\t\t  ");

    lex_test!(
        newline_test,
        "this text is on line one\nthis is on line two\n    this is indented by 4 spaces"
    );

    lex_test!(unclosed_string, "\"hello world");
    lex_test!(unclosed_string_newline, "\"hello world\ngoodbye\"");

    lex_test!(single_cr_linebreak, "testing\rbad newline");

    lex_test!(invalid_char, "◑﹏◐");
}
