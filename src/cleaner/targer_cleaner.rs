use std::error::Error;
use std::fs;
use std::path::Path;

use crate::scanner::rust_project::RustProject;
/// Utility for cleaning up target directories
pub struct TargetCleaner;

impl TargetCleaner {
    /// Clean up target directories for the selected projects
    pub fn clean_selected_projects(
        projects: &[RustProject],
        selected_indices: &[bool],
        dry_run: bool,
    ) -> Result<CleanupResult, Box<dyn Error>> {
        let mut total_freed = 0u64;
        let mut errors = Vec::new();

        for (i, project) in projects.iter().enumerate() {
            if selected_indices.get(i).copied().unwrap_or(false) {
                if let Some(ref target_info) = project.target_info {
                    let _project_name = &project.name;
                    let target_path = &target_info.path;
                    let size = target_info.size_bytes;

                    if dry_run {
                        // Just simulate deletion in dry run mode
                        println!(
                            "Would delete: {} ({})",
                            target_path.display(),
                            format_bytes(size)
                        );
                        total_freed += size;
                    } else {
                        // Actually delete the target directory
                        match Self::delete_target_directory(target_path) {
                            Ok(_) => {
                                println!(
                                    "Deleted: {} ({})",
                                    target_path.display(),
                                    format_bytes(size)
                                );
                                total_freed += size;
                            }
                            Err(e) => {
                                let error =
                                    format!("Failed to delete {}: {}", target_path.display(), e);
                                eprintln!("Error: {}", error);
                                errors.push(error);
                            }
                        }
                    }
                }
            }
        }

        Ok(CleanupResult {
            total_freed,
            errors,
        })
    }

    /// Delete a target directory and all its contents
    fn delete_target_directory(target_path: &Path) -> Result<(), Box<dyn Error>> {
        // Check if the path exists before trying to delete
        if !target_path.exists() {
            return Ok(()); // Already deleted
        }

        // Remove the directory and all its contents
        fs::remove_dir_all(target_path)?;
        Ok(())
    }
}

/// Result of a cleanup operation
#[derive(Debug)]
pub struct CleanupResult {
    /// Total bytes freed
    pub total_freed: u64,
    /// List of errors that occurred
    pub errors: Vec<String>,
}

/// Format bytes into a human-readable string
fn format_bytes(bytes: u64) -> String {
    const UNITS: &[&str] = &["B", "KB", "MB", "GB", "TB"];
    const THRESHOLD: f64 = 1024.0;

    if bytes == 0 {
        return "0 B".to_string();
    }

    let bytes_f = bytes as f64;
    let unit_index = (bytes_f.log10() / THRESHOLD.log10()).floor() as usize;
    let unit_index = unit_index.min(UNITS.len() - 1);
    let scaled = bytes_f / THRESHOLD.powi(unit_index as i32);

    if unit_index == 0 {
        format!("{} {}", bytes, UNITS[unit_index])
    } else {
        format!("{:.2} {}", scaled, UNITS[unit_index])
    }
}
