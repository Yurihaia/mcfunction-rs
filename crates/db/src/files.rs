//! The link between the file system and the database.
//! Each datapack is just a map of files to their ID in the database.
//! These datapacks can be standard datapack in the folder in a world,
//! or they can be zipped datapacks whose actual files exist in an outside location

use relative_path::{RelativePath, RelativePathBuf};

use std::collections::HashMap;

/// A copyable index into a file arena
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct FileId(pub u32);

/// A copyable index into the datapack arena
#[derive(Debug, Copy, Clone, PartialEq, Eq, Hash)]
pub struct DatapackId(pub u32);

/// A definition of a single datapack
#[derive(Debug)]
pub struct Datapack {
    archive: bool,
    files: HashMap<RelativePathBuf, FileId>,
}

impl Datapack {
    pub fn new(archive: bool) -> Self {
        Datapack {
            archive,
            files: HashMap::new(),
        }
    }

    pub fn is_archive(&self) -> bool {
        self.archive
    }

    pub fn files(&self) -> impl Iterator<Item = FileId> + '_ {
        self.files.values().copied()
    }

    pub fn insert(&mut self, path: RelativePathBuf, id: FileId) {
        self.files.insert(path, id);
    }

    pub fn get(&mut self, path: &RelativePath) -> Option<FileId> {
        self.files.get(path).copied()
    }

    pub fn remove(&mut self, path: &RelativePath) -> Option<FileId> {
        self.files.remove(path)
    }
}
