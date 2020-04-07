use super::McTokenKind::{self, *};
use mcfunction_parse::{LineCol, Span, Token};

pub const PUNCT: &[(&str, McTokenKind)] = &[
    // Double
    ("..", DotDot),
    ("<=", Lte),
    (">=", Gte),
    ("><", Swap),
    ("+=", AddAssign),
    ("-=", SubAssign),
    ("*=", MulAssign),
    ("/=", DivAssign),
    ("%=", ModAssign),
    // Single
    (",", Comma),
    (".", Dot),
    (":", Colon),
    (";", Semicolon),
    ("@", At),
    ("!", Excl),
    ("=", Eq),
    ("<", Lt),
    (">", Gt),
    ("/", Slash),
    ("~", Tilde),
    ("^", Caret),
    ("+", Plus),
    ("-", Dash),
    ("#", Hash),
    // Delimiters
    ("{", LCurly),
    ("}", RCurly),
    ("[", LBracket),
    ("]", RBracket),
];

pub fn tokenize_str(src: &str) -> Vec<Vec<Token<McTokenKind>>> {
    let mut off = 0;
    let mut lines = vec![];
    let mut line_buf = vec![];
    let mut line = 0;
    let mut col = 0;
    'out: loop {
        let start = LineCol::new(line, col);
        if off >= src.len() {
            break;
        }
        for (s, t) in PUNCT {
            if src[off..].starts_with(s) {
                col += s.len();
                line_buf.push(Token::new(
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
                .take_while(|(_, c)| c.is_ascii_alphabetic() || *c == '_')
                .last()
                .unwrap();
            let end = end + endch.len_utf8();
            col += end;
            line_buf.push(Token::new(
                Word,
                Span::new(start, LineCol::new(line, col)),
                off,
                off + end,
            ));
            off += end;
        } else if cn.is_ascii_digit() {
            let (end, endch) = src[off..]
                .char_indices()
                .take_while(|(_, c)| c.is_ascii_digit())
                .last()
                .unwrap();
            let end = end + endch.len_utf8();
            col += end;
            line_buf.push(Token::new(
                Digits,
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
            for ch in chars {
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
            line_buf.push(Token::new(
                QuotedString,
                Span::new(start, LineCol::new(line, col)),
                off,
                off + offset,
            ));
            off += offset;
        } else if cn == '\r' {
            col = 0;
            line += 1;
            if let Some('\n') = src[off..].chars().nth(1) {
                line_buf.push(Token::new(
                    Eof,
                    Span::new(start, LineCol::new(line, col)),
                    off,
                    off + 2,
                ));
                lines.push(line_buf);
                line_buf = vec![];
                off += 2;
            } else {
                line_buf.push(Token::new(
                    Eof,
                    Span::new(start, LineCol::new(line, col)),
                    off,
                    off + 1,
                ));
                lines.push(line_buf);
                line_buf = vec![];
                off += 1;
            }
        } else if cn == '\n' {
            col = 0;
            line += 1;
            line_buf.push(Token::new(
                Eof,
                Span::new(start, LineCol::new(line, col)),
                off,
                off + 1,
            ));
            lines.push(line_buf);
            line_buf = vec![];
            off += 1;
        } else if cn.is_whitespace() {
            let (end, endch) = src[off..]
                .char_indices()
                .take_while(|(_, c)| c.is_whitespace() && *c != '\r' && *c != '\n')
                .last()
                .unwrap();
            let end = end + endch.len_utf8();
            col += end;
            line_buf.push(Token::new(
                Whitespace,
                Span::new(start, LineCol::new(line, col)),
                off,
                off + end,
            ));
            off += end;
        } else {
            let len = cn.len_utf8();
            col += len;
            line_buf.push(Token::new(
                Invalid,
                Span::new(start, LineCol::new(line, col)),
                off,
                off + len,
            ));
            off += len;
        }
    }
    line_buf.push(Token::new(
        Eof,
        Span::new(LineCol::new(line, col), LineCol::new(line, col)),
        off,
        off,
    ));
    lines.push(line_buf);
    return lines;
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

    lex_test!(ident, "sOmE_IdEntiFier");
    lex_test!(punct_single, "%");
    lex_test!(punct_multiple, "<=><%.-");
    lex_test!(lit_string, r#""hello""#);
    lex_test!(lit_string_escapes, r#""hello \"\q\u\o\t\e\d\\\" world""#);
    lex_test!(digit_literal, "120394");
    lex_test!(integer, "-2147483648");
    lex_test!(float, "0.5772156649");
    lex_test!(group_simple, "{literal}");
    lex_test!(group_complex, "{.\"hello\" \n ident}");
    lex_test!(group_nested, "{ { { a } b c } d }");
    lex_test!(multi_top, "identifier 1023:7");
    lex_test!(whitespace, "   \t    \t\t\t  ");

    lex_test!(
        command_execute,
        r#"execute as @e[tag="foo",type=minecraft:pig,nbt={a:123b,c:hello,d:[B;1,2,3]}] run say hi"#
    );

    lex_test!(
        newline_test,
        "this text is on line one\nthis is on line two\n    this is indented by 4 spaces"
    );

    lex_test!(unclosed_group, "{ unclosed group");

    lex_test!(unclosed_string, "\"hello world");
    lex_test!(unclosed_string_newline, "\"hello world\ngoodbye\"");

    lex_test!(single_cr_linebreak, "testing\rbad newline");
}
