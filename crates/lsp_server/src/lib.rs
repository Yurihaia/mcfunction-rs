pub mod datapack;
pub mod manager;
pub mod reports;

use crossbeam_channel::{unbounded, Receiver};
use ra_vfs::{Filter, RelativePath, RootEntry, Vfs, VfsTask};
use std::path::PathBuf;

pub struct WorldState {
    vfs: Vfs,
    roots: Vec<PathBuf>,
    receiver: Receiver<VfsTask>,
}

pub struct FileFilter;

impl Filter for FileFilter {
    fn include_file(&self, rel: &RelativePath) -> bool {
        true
    }
    fn include_dir(&self, rel: &RelativePath) -> bool {
        true
    }
}

impl WorldState {
    pub fn new(roots: Vec<PathBuf>) -> Self {
        let (sender, receiver) = unbounded::<VfsTask>();
        let vfs_roots = roots
            .iter()
            .cloned()
            .map(|v| RootEntry::new(v, Box::new(FileFilter)))
            .collect();
        let (vfs, _) = Vfs::new(
            vfs_roots,
            Box::from(move |v| sender.send(v).unwrap()),
            ra_vfs::Watch(true),
        );
        WorldState {
            vfs,
            roots,
            receiver,
        }
    }
}
