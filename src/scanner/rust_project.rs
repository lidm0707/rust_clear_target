use crate::scanner::target_finder::TargetInfo;
use std::error::Error;
use std::path::{Path, PathBuf};

#[derive(Debug, Clone)]
pub struct RustProject {
    /// Path to the project directory (containing Cargo.toml)
    pub path: PathBuf,
    /// Name of the project derived from Cargo.toml
    pub name: String,
    /// Information about the target directory
    pub target_info: Option<TargetInfo>,
}

impl RustProject {
    /// Creates a RustProject from a directory path containing Cargo.toml
    pub fn from_path(path: &Path) -> Result<Self, Box<dyn Error>> {
        if !path.exists() {
            return Err(format!("Project path does not exist: {:?}", path).into());
        }

        let cargo_toml = path.join("Cargo.toml");
        if !cargo_toml.exists() {
            return Err(format!("Cargo.toml not found in: {:?}", path).into());
        }

        let name = Self::extract_project_name(&cargo_toml)?;

        Ok(Self {
            path: path.to_path_buf(),
            name,
            target_info: None,
        })
    }

    /// Adds target information to the project
    pub fn with_target_info(mut self, target_info: TargetInfo) -> Self {
        self.target_info = Some(target_info);
        self
    }

    /// Extracts the project name from Cargo.toml
    fn extract_project_name(cargo_toml: &Path) -> Result<String, Box<dyn Error>> {
        let content = std::fs::read_to_string(cargo_toml)?;

        // Simple parsing to extract the name from [package] section
        // This is a basic implementation - in a real scenario, you'd use toml crate
        let lines: Vec<&str> = content.lines().collect();
        let mut in_package = false;

        for line in lines {
            let trimmed = line.trim();

            if trimmed == "[package]" {
                in_package = true;
                continue;
            }

            if trimmed.starts_with('[') && trimmed != "[package]" {
                in_package = false;
                continue;
            }

            if in_package && trimmed.starts_with("name") {
                if let Some(name_part) = trimmed.split('=').nth(1) {
                    let name = name_part.trim().trim_matches('"').trim_matches('\'');
                    return Ok(name.to_string());
                }
            }
        }

        // Fallback to directory name if name not found
        if let Some(parent) = cargo_toml.parent() {
            if let Some(dir_name) = parent.file_name() {
                if let Some(name_str) = dir_name.to_str() {
                    return Ok(name_str.to_string());
                }
            }
        }

        Err("Could not determine project name".into())
    }

    /// Returns the path to the project's target directory
    #[allow(dead_code)]
    pub fn target_path(&self) -> Option<PathBuf> {
        self.path.join("target").into()
    }
}
