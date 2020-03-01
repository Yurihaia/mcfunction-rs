pub mod modules;
pub mod syntax;

use crate::syntax::grammar::file;
use mcfunction_db::{FileId, FsDatabase, NamespaceId};
use mcfunction_parse::{parser::Parser, Ast};
use modules::{ModuleTree, Submodule};
use salsa;
use std::sync::Arc;
use syntax::{
    cst::{File, Item, Node},
    group::NdGroupType,
    lexer, NbtdocLang,
};
use util::arena::{Arena, RawId};

#[salsa::query_group(NbtdocFileDatabaseStorage)]
pub trait NbtdocFileDatabase: std::fmt::Debug + FsDatabase {
    fn parse(&self, id: FileId) -> Arc<Ast<Arc<str>, NbtdocLang>>;
}

#[salsa::query_group(NbtdocItemStorage)]
pub trait NbtdocItemDatabase: std::fmt::Debug + NbtdocFileDatabase {
    fn file_items(&self, id: FileId) -> Arc<FileItems>;

    #[salsa::invoke(modules::submodules)]
    fn submodules(&self, id: FileId) -> Arc<Vec<Submodule>>;
}

#[salsa::query_group(NbtdocModuleStorage)]
pub trait NbtdocModuleDatabase: std::fmt::Debug + NbtdocItemDatabase {
    #[salsa::invoke(modules::module_tree)]
    fn module_tree(&self, root: NamespaceId) -> Arc<ModuleTree>;
}

#[derive(Debug, PartialEq, Eq)]
pub struct FileItems {
    source: FileId,
    items: Arena<ItemId, Item<Node>>,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct ItemId(RawId);
util::arena_id!(ItemId);

impl FileItems {
    pub fn items(&self) -> &Arena<ItemId, Item<Node>> {
        &self.items
    }

    pub fn source(&self) -> FileId {
        self.source
    }
}

fn file_items(db: &impl NbtdocItemDatabase, id: FileId) -> Arc<FileItems> {
    let tree: File<Node> = match Ast::cst_root(db.parse(id)) {
        Ok(v) => v,
        Err(_) => {
            return Arc::new(FileItems {
                source: id,
                items: Arena::new(),
            })
        }
    };
    let mut out = Arena::new();
    for item in tree.items() {
        out.push(item.into_arc());
    }
    Arc::new(FileItems {
        source: id,
        items: out,
    })
}

fn parse(db: &impl NbtdocFileDatabase, id: FileId) -> Arc<Ast<Arc<str>, NbtdocLang>> {
    let text = db.file_text(id);
    let tokens = lexer::tokenize_str(&text);
    assert!(!tokens.is_empty(), "Token stream is empty");
    let mut parser = Parser::new(&tokens, &text, NdGroupType::File, true);
    file(&mut parser);
    Arc::new(parser.build(true).retype_src_with(|_| text.clone()))
}
