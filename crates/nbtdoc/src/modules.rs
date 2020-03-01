use crate::ItemId;
use crate::{NbtdocItemDatabase, NbtdocModuleDatabase};
use mcfunction_db::NamespaceId;
use mcfunction_db::{DataType, FileId};
use mcfunction_parse::ast::CstNode;
use relative_path::RelativePathBuf;
use std::sync::Arc;
use util::arena::{Arena, RawId};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleTree {
    mods: Arena<ModuleId, Module>,
    links: Arena<ModuleLinkId, ModuleLink>,
    root: ModuleId,
}

#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct ModuleId(RawId);
util::arena_id!(ModuleId);
#[derive(Copy, Clone, Hash, PartialEq, Eq)]
pub struct ModuleLinkId(RawId);
util::arena_id!(ModuleLinkId);

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum Module {
    Missing,
    Present {
        parent: Option<ModuleLinkId>,
        children: Vec<ModuleLinkId>,
    },
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ModuleLink {
    name: String,
    owner: ModuleId,
    child: ModuleId,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Submodule {
    name: String,
    source: ItemId,
}

pub fn submodules(db: &impl NbtdocItemDatabase, id: FileId) -> Arc<Vec<Submodule>> {
    let items = db.file_items(id);
    Arc::new(
        items
            .items()
            .entries()
            .filter_map(|v| {
                Some(Submodule {
                    name: String::from(v.0.mod_decl()?.name()?.view().string()),
                    source: v.1,
                })
            })
            .collect(),
    )
}

pub fn module_tree(db: &impl NbtdocModuleDatabase, namespace_id: NamespaceId) -> Arc<ModuleTree> {
    if let Some(root) = db.namespace_file(
        namespace_id,
        DataType::Nbtdoc,
        RelativePathBuf::from("mod.nbtdoc"),
    ) {
        let mut modules: Arena<ModuleId, Module> = Arena::new();
        let mut links: Arena<ModuleLinkId, ModuleLink> = Arena::new();
        // ( Submodule to process, Module File Id, Parent module )
        let mut to_process: Vec<(Submodule, Option<FileId>, ModuleId)> = vec![];
        let root_ind = modules.push(Module::Present {
            children: vec![],
            parent: None,
        });
        to_process.extend(db.submodules(root).iter().cloned().map(|v| {
            let fid = db
                .namespace_file(
                    namespace_id,
                    DataType::Nbtdoc,
                    RelativePathBuf::from(format!("{}.nbtdoc", &v.name)),
                )
                .or_else(|| {
                    db.namespace_file(
                        namespace_id,
                        DataType::Nbtdoc,
                        RelativePathBuf::from(format!("{}/mod.nbtdoc", &v.name)),
                    )
                });
            (v, fid, root_ind)
        }));
        while let Some((m, id, parent)) = to_process.pop() {
            if let Some(id) = id {
                let path = db.path(id).join(&m.name);
                let mlink = links.push(ModuleLink {
                    name: m.name,
                    // will get updated later
                    child: parent,
                    owner: parent,
                });
                let mod_ind = modules.push(Module::Present {
                    children: vec![],
                    parent: Some(mlink),
                });
                links[mlink].child = mod_ind;
                match &mut modules[parent] {
                    Module::Missing => (),
                    Module::Present { children, .. } => {
                        children.push(mlink);
                    }
                }
                if let Some(file) = db.namespace_file(namespace_id, DataType::Nbtdoc, path) {
                    to_process.extend(db.submodules(file).iter().cloned().map(|v| {
                        let path = db.path(id);
                        let name = path.file_name().expect("Paths should never end in `..`");
                        let fid = if name == "mod.nbtdoc" {
                            let rootdir = path.parent().expect("Path always points to a file");
                            db.namespace_file(
                                namespace_id,
                                DataType::Nbtdoc,
                                rootdir.join(format!("{}.nbtdoc", &v.name)),
                            )
                            .or_else(|| {
                                db.namespace_file(
                                    namespace_id,
                                    DataType::Nbtdoc,
                                    rootdir.join(format!("{}/mod.nbtdoc", &v.name)),
                                )
                            })
                        } else {
                            assert_eq!(path.extension(), Some("nbtdoc"));
                            let mut rootdir = path.with_extension("");
                            rootdir.push(format!("{}/mod.nbtdoc", &v.name));
                            db.namespace_file(namespace_id, DataType::Nbtdoc, rootdir)
                        };
                        (v, fid, mod_ind)
                    }));
                }
            } else {
                let mlink = links.push(ModuleLink {
                    name: m.name,
                    child: parent,
                    owner: parent,
                });
                let mod_ind = modules.push(Module::Missing);
                links[mlink].child = mod_ind;
                match &mut modules[parent] {
                    Module::Missing => (),
                    Module::Present { children, .. } => {
                        children.push(mlink);
                    }
                }
            }
        }
        Arc::new(ModuleTree {
            links,
            mods: modules,
            root: root_ind,
        })
    } else {
        let mut mods = Arena::new();
        let root = mods.push(Module::Missing);
        Arc::new(ModuleTree {
            links: Arena::new(),
            mods,
            root,
        })
    }
}
