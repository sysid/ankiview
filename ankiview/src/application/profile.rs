// src/application/profile.rs
use anyhow::Result;
use std::path::PathBuf;

pub trait ProfileLocator {
    fn find_collection_path(&self, profile: Option<&str>) -> Result<PathBuf>;
}
