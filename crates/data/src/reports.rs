use serde::{Deserialize, Serialize};
use util::commands::Commands;

use std::collections::HashMap;

pub type BlockList = HashMap<String, BlockData>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockData {
    #[serde(default)]
    pub properties: HashMap<String, Vec<String>>,
    pub states: Vec<BlockState>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct BlockState {
    pub id: usize,
    #[serde(default)]
    pub properties: HashMap<String, String>,
    #[serde(default)]
    pub default: bool,
}

pub type RegistryList = HashMap<String, RegistryInfo>;

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistryInfo {
    #[serde(default)]
    pub default: Option<String>,
    pub protocol_id: usize,
    pub entries: HashMap<String, RegistryEntry>,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct RegistryEntry {
    pub protocol_id: usize,
}

#[derive(Serialize, Deserialize, Debug, Clone)]
pub struct VersionData {
    pub registries: RegistryList,
    pub blocks: BlockList,
    pub commands: Commands,
}
