use std::{
    error::Error,
    io::Write,
    path::{Path, PathBuf},
};

use crate::scanner::{rust_project::RustProject, target_finder::TargetFinder};

pub struct RustProjectScanner {
    search_paths: Vec<PathBuf>,
    exclude_patterns: Vec<String>,
    ignore_paths: Vec<PathBuf>,
}

impl RustProjectScanner {
    /// Creates a new scanner with the specified search paths and exclusion patterns
    #[allow(dead_code)]
    pub fn new(
        search_paths: &[PathBuf],
        exclude_patterns: &[String],
    ) -> Result<Self, Box<dyn Error>> {
        Self::new_with_ignores(search_paths, exclude_patterns, &[])
    }

    /// Creates a new scanner with the specified search paths, exclusion patterns, and ignore paths
    pub fn new_with_ignores(
        search_paths: &[PathBuf],
        exclude_patterns: &[String],
        ignore_paths: &[PathBuf],
    ) -> Result<Self, Box<dyn Error>> {
        // Validate search paths exist
        for path in search_paths {
            if !path.exists() {
                return Err(format!("Search path does not exist: {:?}", path).into());
            }
        }

        Ok(Self {
            search_paths: search_paths.to_vec(),
            exclude_patterns: exclude_patterns.to_vec(),
            ignore_paths: ignore_paths.to_vec(),
        })
    }

    /// Scans all configured paths for Rust projects with target directories
    pub fn find_projects(&self) -> Result<Vec<RustProject>, Box<dyn Error>> {
        let mut projects = Vec::new();

        // Filter out paths that should be ignored
        let filtered_paths: Vec<&PathBuf> = self
            .search_paths
            .iter()
            .filter(|path| !self.is_ignored_path(path))
            .collect();

        println!(
            "Searching in {} directories ({} ignored)...",
            filtered_paths.len(),
            self.search_paths.len() - filtered_paths.len()
        );

        for (i, path) in filtered_paths.iter().enumerate() {
            // Check if this search path should be ignored
            if self.is_ignored_path(path) {
                println!("Skipping ignored path: {}", path.display());
                continue;
            }

            println!(
                "Scanning {}/{}: {}",
                i + 1,
                filtered_paths.len(),
                path.display()
            );
            let found_projects = self.scan_path(path)?;
            println!(
                "Found {} Rust projects in {}",
                found_projects.len(),
                path.display()
            );
            projects.extend(found_projects);
        }

        Ok(projects)
    }

    /// Scans a single path for Rust projects
    fn scan_path(&self, path: &Path) -> Result<Vec<RustProject>, Box<dyn Error>> {
        let mut projects = Vec::new();
        let mut directories_scanned = 0;
        let mut cargo_files_found = 0;

        // Use walkdir to traverse the directory tree
        for entry in walkdir::WalkDir::new(path)
            .into_iter()
            .filter_entry(|e| {
                !is_excluded(e.path(), &self.exclude_patterns) && !self.is_ignored_path(e.path())
            })
            .filter_map(Result::ok)
        {
            directories_scanned += 1;

            // Show progress for every 1000 directories scanned
            if directories_scanned % 1000 == 0 {
                print!(".");
                std::io::stdout().flush().unwrap();
            }

            if entry.file_name() == "Cargo.toml" {
                cargo_files_found += 1;
                let cargo_path = entry.path();
                let project_path = cargo_path.parent().unwrap_or(cargo_path);

                if let Ok(project) = RustProject::from_path(project_path) {
                    if let Ok(target_info) = TargetFinder::find_target_info(project_path) {
                        let project_with_target = project.with_target_info(target_info);
                        projects.push(project_with_target);
                    }
                }
            }
        }

        println!();
        println!(
            "Scanned {} directories, found {} Cargo.toml files",
            directories_scanned, cargo_files_found
        );

        Ok(projects)
    }
}

/// Checks if a path should be excluded from scanning
fn is_excluded(path: &Path, patterns: &[String]) -> bool {
    let path_str = path.to_string_lossy();

    for pattern in patterns {
        if path_str.contains(pattern) {
            return true;
        }
    }

    false
}

impl RustProjectScanner {
    /// Checks if a path should be ignored based on the ignore_paths list
    fn is_ignored_path(&self, path: &Path) -> bool {
        // Check if path is exactly in the ignore list
        for ignore_path in &self.ignore_paths {
            if path
                .to_string_lossy()
                .as_ref()
                .contains(ignore_path.to_string_lossy().as_ref())
            {
                return true;
            }

            // Check if path is a child of any ignored path
            // We need to normalize paths first
            let normalized_path = path.to_string_lossy();
            let normalized_ignore = ignore_path.to_string_lossy();

            // Add trailing slash to avoid matching similar names
            let ignore_with_slash = format!("{}/", normalized_ignore);

            if normalized_path.starts_with(&ignore_with_slash) {
                return true;
            }

            // Also check if normalized path starts with normalized ignore
            // and either they are equal or next character is a separator
            if normalized_path.starts_with(&normalized_ignore.as_ref()) {
                if normalized_path.len() == normalized_ignore.len() {
                    return true; // Exact match
                }

                // Check if the next character after the match is a separator
                if normalized_path.chars().nth(normalized_ignore.len()) == Some('/')
                    || normalized_path.chars().nth(normalized_ignore.len())
                        == Some(std::path::MAIN_SEPARATOR)
                {
                    return true;
                }
            }
        }

        false
    }
}
