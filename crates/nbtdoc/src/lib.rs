pub mod syntax;

use crate::syntax::grammar::file;
use mcfunction_db::{FileId, FsDatabase};
use mcfunction_parse::{parser::Parser, Ast};
use salsa;
use std::sync::Arc;
use syntax::{lexer, NbtdocLang};

#[salsa::query_group(NbtdocFileDatabaseStorage)]
pub trait NbtdocFileDatabase: std::fmt::Debug + FsDatabase {
    #[salsa::invoke(parse_nbtdoc)]
    fn parse(&self, id: FileId) -> Arc<Ast<Arc<str>, NbtdocLang>>;
}

fn parse_nbtdoc(db: &impl NbtdocFileDatabase, id: FileId) -> Arc<Ast<Arc<str>, NbtdocLang>> {
    let text = db.file_text(id);
    let tokens = lexer::tokenize_str(&text);
    assert!(!tokens.is_empty(), "Token stream is empty");
    let mut parser = Parser::new(&tokens, &text);
    file(&mut parser);
    Arc::new(parser.build(true).retype_src_with(|_| text.clone()))
}
