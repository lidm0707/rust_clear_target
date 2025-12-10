use std::error::Error;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::{Duration, SystemTime};

/// Information about a target directory
#[derive(Debug, Clone)]
pub struct TargetInfo {
    /// Path to the target directory
    pub path: PathBuf,
    /// Total size in bytes
    pub size_bytes: u64,
    /// Last modification time (more reliable than access time)
    pub last_accessed: SystemTime,
    /// Whether the directory is considered stale (not accessed for a while)
    pub is_stale: bool,
}

/// Utility for finding and analyzing target directories
pub struct TargetFinder;

impl TargetFinder {
    /// Finds and analyzes the target directory for a Rust project
    pub fn find_target_info(project_path: &Path) -> Result<TargetInfo, Box<dyn Error>> {
        let target_path = project_path.join("target");

        if !target_path.exists() || !target_path.is_dir() {
            return Err(format!("Target directory not found: {:?}", target_path).into());
        }

        let size_bytes = Self::calculate_directory_size(&target_path)?;
        let last_accessed = Self::get_last_accessed_time(&target_path)?;

        // Default to considering it stale (will be updated by analyzer)
        let is_stale = false;

        Ok(TargetInfo {
            path: target_path,
            size_bytes,
            last_accessed,
            is_stale,
        })
    }

    /// Calculates the total size of a directory recursively with optimizations for large directories
    fn calculate_directory_size(dir_path: &Path) -> Result<u64, Box<dyn Error>> {
        let mut total_size = 0u64;
        let mut file_count = 0;

        // Optimized walkdir configuration
        for entry in walkdir::WalkDir::new(dir_path)
            .follow_links(false) // Don't follow symlinks
            .max_open(128) // Limit file descriptors
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    total_size += metadata.len();
                    file_count += 1;

                    // For directories with many files, avoid scanning everything
                    // Estimate size based on sample for very large directories
                    if file_count > 10000 {
                        // Calculate average file size so far and estimate
                        let avg_size = if file_count > 0 {
                            total_size / file_count as u64
                        } else {
                            0
                        };

                        // Estimate total based on directory entry count (which is faster)
                        if let Ok(dir_entry_count) = Self::count_directory_entries(dir_path) {
                            return Ok(avg_size * dir_entry_count);
                        }
                    }
                }
            }
        }

        Ok(total_size)
    }

    /// Gets the last accessed time for a directory or its most recent file
    fn get_last_accessed_time(dir_path: &Path) -> Result<SystemTime, Box<dyn Error>> {
        // First try to get modification time, which is more reliably updated than access time
        let mut last_modified = fs::metadata(dir_path)?.modified()?;
        let mut files_checked = 0;
        let mut found_older_file = false;

        // Optimized walkdir configuration
        for entry in walkdir::WalkDir::new(dir_path)
            .follow_links(false) // Don't follow symlinks
            .max_open(128) // Limit file descriptors
            .into_iter()
            .filter_map(Result::ok)
        {
            if entry.file_type().is_file() {
                if let Ok(metadata) = entry.metadata() {
                    // Try to get modification time first, as it's more reliable
                    if let Ok(modified) = metadata.modified() {
                        // Update if we found an older file (more representative of last use)
                        if modified < last_modified && !found_older_file {
                            last_modified = modified;
                            found_older_file = true;
                        }

                        // Keep track of the oldest file we've found
                        if modified < last_modified {
                            last_modified = modified;
                        }
                    }
                }

                files_checked += 1;

                // Limit the number of files we check for performance
                // After checking 100 files, we should have a reasonable sample
                if files_checked > 100 {
                    break;
                }
            }
        }

        // If we couldn't find a reliable timestamp, use a default (30 days ago)
        if !found_older_file {
            let default_age = SystemTime::now() - Duration::from_secs(30 * 24 * 60 * 60);
            return Ok(default_age);
        }

        Ok(last_modified)
    }

    /// Counts the number of entries in a directory (faster than walking all files)
    fn count_directory_entries(dir_path: &Path) -> Result<u64, Box<dyn Error>> {
        let mut count = 0;

        if let Ok(entries) = fs::read_dir(dir_path) {
            for _ in entries.filter_map(Result::ok) {
                count += 1;

                // Cap at a reasonable limit
                if count > 100000 {
                    break;
                }
            }
        }

        Ok(count)
    }

    /// Checks if a target directory is considered stale based on the given threshold
    pub fn is_stale(target_info: &TargetInfo, threshold: Duration) -> Result<bool, Box<dyn Error>> {
        let now = SystemTime::now();
        let time_diff = now
            .duration_since(target_info.last_accessed)
            .unwrap_or_else(|_| Duration::from_secs(0));

        Ok(time_diff >= threshold)
    }

    /// Updates a TargetInfo to determine if it's stale based on the threshold
    pub fn update_stale_status(
        target_info: &mut TargetInfo,
        threshold: Duration,
    ) -> Result<(), Box<dyn Error>> {
        target_info.is_stale = Self::is_stale(target_info, threshold)?;
        Ok(())
    }
}
