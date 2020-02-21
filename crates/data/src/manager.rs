use crate::{
    reports::{BlockData, BlockList, RegistryInfo, RegistryList, VersionData},
    ReportDatabase,
};

use std::fs::{self, File};
use std::io;
use std::path::{Path, PathBuf};
use std::process::Command;

use util::commands::{CommandNode, Commands};

use relative_path::RelativePathBuf;

#[derive(Debug)]
pub struct DataManager {
    global: VersionData,
    temp_dir: PathBuf,
}

#[cfg(not(target_os = "macos"))]
const MC_DIR: &str = ".minecraft";

// Because Mac is special apparently
#[cfg(target_os = "macos")]
const MC_DIR: &str = "minecraft";

impl DataManager {
    pub fn with_dir(
        server_path: impl AsRef<Path>,
        java_path: impl AsRef<Path>,
        nbtdoc_path: impl AsRef<Path>,
        dir: impl AsRef<Path>,
    ) -> io::Result<Self> {
        let dir = dir.as_ref();
        if !dir.exists() {
            fs::create_dir(&dir)?;
            let mut child = Command::new(java_path.as_ref())
                .arg("-cp")
                .arg(server_path.as_ref())
                .args(&["net.minecraft.data.Main", "--reports", "--output"])
                .arg(&dir)
                .current_dir(&dir)
                .spawn()?;
            child.wait()?;
            // Extract vanilla datapack
            let mut archive = zip::ZipArchive::new(File::open(server_path)?)?;
            let extr_dir = dir.join("datapacks/vanilla");
            for x in 0..archive.len() {
                let mut file = archive.by_index(x).unwrap();
                if !file.is_file() {
                    continue;
                }
                let output = RelativePathBuf::from_path(file.sanitized_name()).unwrap();
                if output.starts_with("data") || output == "pack.mcmeta" {
                    let out_path = output.to_path(&extr_dir);
                    if let Some(v) = out_path.parent() {
                        if !v.exists() {
                            fs::create_dir_all(v)?;
                        }
                    }
                    let mut out = File::create(out_path)?;
                    io::copy(&mut file, &mut out)?;
                }
            }
            let nbtdoc_copy = extr_dir.join("data/minecraft/nbtdoc");
            fs::create_dir_all(&nbtdoc_copy)?;
            copy_recursive(nbtdoc_path, nbtdoc_copy)?;
        }
        // Read reports
        let registries: RegistryList =
            serde_json::from_reader(File::open(dir.join("reports/registries.json"))?)?;
        let blocks: BlockList =
            serde_json::from_reader(File::open(dir.join("reports/blocks.json"))?)?;
        let commands: CommandNode =
            serde_json::from_reader(File::open(dir.join("reports/commands.json"))?)?;

        Ok(DataManager {
            global: VersionData {
                registries,
                blocks,
                commands: Commands::generate(commands),
            },
            temp_dir: PathBuf::from(dir),
        })
    }

    pub fn create(
        server_path: impl AsRef<Path>,
        java_path: impl AsRef<Path>,
        nbtdoc_path: impl AsRef<Path>,
        version: &str,
    ) -> io::Result<Self> {
        let temp_dir = {
            let mut dir = if let Some(data_dir) = dirs::data_dir() {
                let joined = data_dir.join(MC_DIR);
                if joined.exists() {
                    joined
                } else {
                    data_dir
                }
            } else {
                std::env::temp_dir()
            };
            dir.push(format!(".mcf.lsp.cache/{}", version));
            if !dir.parent().unwrap().exists() {
                fs::create_dir_all(dir.parent().unwrap())?;
            }
            dir
        };
        Self::with_dir(server_path, java_path, nbtdoc_path, temp_dir)
    }

    pub fn global_data(&self) -> &VersionData {
        &self.global
    }

    pub fn cached_datapacks(&self) -> PathBuf {
        self.temp_dir.join("datapacks")
    }

    pub fn vanilla(&self) -> PathBuf {
        let mut p = self.cached_datapacks();
        p.push("vanilla");
        p
    }

    pub fn delete(self) -> io::Result<()> {
        fs::remove_dir_all(self.temp_dir)
    }

    pub fn regenerate(
        &mut self,
        server_path: impl AsRef<Path>,
        java_path: impl AsRef<Path>,
        nbtdoc_path: impl AsRef<Path>,
    ) -> io::Result<()> {
        fs::remove_dir_all(&self.temp_dir)?;
        std::mem::replace(
            self,
            Self::with_dir(server_path, java_path, nbtdoc_path, &self.temp_dir)?,
        );
        Ok(())
    }
}

impl ReportDatabase for DataManager {
    fn block_data(&self, id: &str) -> Option<&BlockData> {
        self.global.blocks.get(id)
    }
    fn registry_data(&self, id: &str) -> Option<&RegistryInfo> {
        self.global.registries.get(id)
    }
    fn commands(&self) -> &Commands {
        &self.global.commands
    }
}

fn copy_recursive(dir: impl AsRef<Path>, to: impl AsRef<Path>) -> io::Result<()> {
    for p in fs::read_dir(&dir)? {
        let p: fs::DirEntry = p?;
        let ft = p.file_type()?;
        if ft.is_dir() {
            let path = to.as_ref().join(p.file_name());
            fs::create_dir(&path)?;
            copy_recursive(p.path(), path)?;
        } else {
            fs::copy(p.path(), to.as_ref().join(p.file_name()))?;
        }
    }
    Ok(())
}

// #[cfg(test)]
// mod tests {
//     use super::*;
//     #[test]
//     fn test_data_manager() {
//         DataManager::create(
//             "D:/Desktop/mcsource/server.jar",
//             "C:/Program Files/Java/jdk-12/bin/java.exe",
//             "D:/Desktop/NbtDoc-Project/mc-nbtdoc-bare/minecraft",
//             "20w06a",
//         )
//         .unwrap();
//     }
// }
