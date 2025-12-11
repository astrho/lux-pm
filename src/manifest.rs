use serde::{Deserialize, Serialize};
use std::collections::HashMap;
use std::fs;
use anyhow::Result;

#[derive(Debug, Deserialize, Serialize)]
pub struct Manifest {
    pub package: Package,
    pub dependencies: Option<HashMap<String, String>>,
}

#[derive(Debug, Deserialize, Serialize)]
pub struct Package {
    pub name: String,
    pub version: String,
}

impl Manifest {
    pub fn from_file(path: &str) -> Result<Self> {
        let content = fs::read_to_string(path)?;
        let manifest: Manifest = toml::from_str(&content)?;
        Ok(manifest)
    }
}
