use crate::reports::{BlockList, RegistryList, VersionData};

use std::fs::{self, File};
use std::path::{Path, PathBuf};
use std::process::{self, Command};
use std::{env, io};

use util::{
    commands::{CommandNode, Commands},
    DropBomb,
};

#[derive(Debug)]
pub struct DataManager {
    dropbomb: DropBomb<&'static str>,
    global: VersionData,
    temp_dir: PathBuf,
}

impl DataManager {
    pub fn with_dir(
        server_path: impl AsRef<Path>,
        java_path: impl AsRef<Path>,
        dir: impl AsRef<Path>,
    ) -> io::Result<Self> {
        let dir = dir.as_ref();
        let mut child = Command::new(java_path.as_ref())
            .arg("-cp")
            .arg(server_path.as_ref())
            .args(&["net.minecraft.data.Main", "--all", "--output"])
            .arg(&dir)
            .spawn()?;
        child.wait()?;
        let registries: RegistryList =
            serde_json::from_reader(File::open(dir.join("reports/registries.json"))?)?;
        let blocks: BlockList =
            serde_json::from_reader(File::open(dir.join("reports/blocks.json"))?)?;
        let commands: CommandNode =
            serde_json::from_reader(File::open(dir.join("reports/commands.json"))?)?;
        Ok(DataManager {
            dropbomb: DropBomb::new("Cannot drop DataHandler without calling 'close'"),
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
        version: &str,
    ) -> io::Result<Self> {
        let temp_dir = {
            let mut dir = env::temp_dir();
            dir.push(format!(
                "yurihaia-mcfunction-data-{}-{}",
                version,
                process::id()
            ));
            fs::create_dir(&dir)?;
            dir
        };
        Self::with_dir(server_path, java_path, temp_dir)
    }

    pub fn global_data(&self) -> &VersionData {
        &self.global
    }

    pub fn datapack_dir(&self) -> &Path {
        &self.temp_dir
    }

    pub fn close(mut self) -> io::Result<()> {
        self.dropbomb.defuse();
        fs::remove_dir_all(self.temp_dir)?;
        Ok(())
    }
}
