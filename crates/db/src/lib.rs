mod files;

pub use files::{Datapack, DatapackId, FileId};

use relative_path::RelativePathBuf;
use salsa;
use std::sync::Arc;

#[salsa::query_group(FsDatabaseStorage)]
pub trait FsDatabase: std::fmt::Debug {
    #[salsa::input]
    fn file_text(&self, id: FileId) -> Arc<str>;

    #[salsa::input]
    fn path(&self, id: FileId) -> RelativePathBuf;

    #[salsa::input]
    fn datapack_id(&self, id: FileId) -> DatapackId;

    #[salsa::input]
    fn datapack(&self, id: DatapackId) -> Arc<Datapack>;
}
