#![deny(unsafe_code)]
pub mod manager;
pub mod reports;

use reports::{BlockData, RegistryInfo};
use util::commands::Commands;

// We'll pretend this is a salsa database, but it doesn't need to be incremental
pub trait ReportDatabase: std::fmt::Debug {
    fn block_data(&self, id: &str) -> Option<&BlockData>;

    fn registry_data(&self, id: &str) -> Option<&RegistryInfo>;

    fn commands(&self) -> &Commands;
}
