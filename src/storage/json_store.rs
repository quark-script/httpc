use anyhow::{Context, Result};
use std::fs;
use std::path::PathBuf;

use crate::models::Workspace;

const APP_DIR: &str = ".httpc";
const PROJECTS_FILE: &str = "projects.json";

pub struct JsonStore {
    base_path: PathBuf,
}

impl JsonStore {
    pub fn new() -> Result<Self> {
        let home = dirs::home_dir().context("Could not find home directory")?;
        let base_path = home.join(APP_DIR);
        Ok(Self { base_path })
    }

    fn ensure_dir(&self) -> Result<()> {
        if !self.base_path.exists() {
            fs::create_dir_all(&self.base_path)
                .context("Failed to create app directory")?;
        }
        Ok(())
    }

    fn projects_path(&self) -> PathBuf {
        self.base_path.join(PROJECTS_FILE)
    }

    pub fn load_workspace(&self) -> Result<Workspace> {
        let path = self.projects_path();

        if !path.exists() {
            return Ok(Workspace::with_sample_data());
        }

        let content = fs::read_to_string(&path)
            .context("Failed to read projects file")?;

        let workspace: Workspace = serde_json::from_str(&content)
            .context("Failed to parse projects file")?;

        Ok(workspace)
    }

    pub fn save_workspace(&self, workspace: &Workspace) -> Result<()> {
        self.ensure_dir()?;

        let content = serde_json::to_string_pretty(workspace)
            .context("Failed to serialize workspace")?;

        fs::write(self.projects_path(), content)
            .context("Failed to write projects file")?;

        Ok(())
    }
}

impl Default for JsonStore {
    fn default() -> Self {
        Self::new().expect("Failed to create JsonStore")
    }
}
