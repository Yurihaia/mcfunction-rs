mod files;

pub use files::{DataType, Datapack, DatapackId, FileId, NamespaceId};
use relative_path::RelativePath;
use relative_path::RelativePathBuf;
use salsa;
use std::{collections::HashSet, sync::Arc};

#[salsa::query_group(FsDatabaseStorage)]
pub trait FsDatabase: std::fmt::Debug {
    #[salsa::input]
    fn file_text(&self, id: FileId) -> Arc<str>;

    #[salsa::input]
    fn path(&self, id: FileId) -> Arc<RelativePath>;

    #[salsa::input]
    fn datapack_id(&self, id: FileId) -> DatapackId;

    #[salsa::input]
    fn datapack(&self, id: DatapackId) -> Arc<Datapack>;

    #[salsa::input]
    fn namespace_id(&self, name: String) -> NamespaceId;

    #[salsa::input]
    fn namespace_file(
        &self,
        id: NamespaceId,
        data: DataType,
        path: RelativePathBuf,
    ) -> Option<FileId>;

    #[salsa::input]
    fn namespace_info(&self, id: NamespaceId, data: DataType) -> Arc<HashSet<RelativePathBuf>>;
}
