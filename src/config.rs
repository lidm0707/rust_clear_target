use serde::Deserialize;
use std::fs;
use std::path::{Path, PathBuf};
use std::time::Duration;

#[derive(Debug, Clone)]
pub struct Config {
    /// Directories to search for Rust projects
    pub search_paths: Vec<PathBuf>,

    /// Patterns to exclude from scanning
    pub exclude_patterns: Vec<String>,
    /// Directories to exclude from scanning
    pub ignore_paths: Vec<PathBuf>,

    /// Age threshold for considering a target directory stale
    pub stale_threshold: Duration,

    /// Number of days to consider a target directory as stale based on last access
    pub last_access_days: u64,

    /// Whether to run in dry-run mode (show what would be deleted without actually deleting)
    pub dry_run: bool,

    /// Whether to be verbose in output
    #[allow(dead_code)]
    pub verbose: bool,

    /// Whether to clear the terminal before starting the UI
    pub clear_terminal: bool,
}

/// TOML configuration structure for deserialization
#[derive(Debug, Deserialize)]
struct CleanerConfig {
    ignore: Option<IgnoreSection>,
    settings: Option<SettingsSection>,
    access: Option<AccessSection>,
}

#[derive(Debug, Deserialize)]
struct IgnoreSection {
    paths: Option<Vec<String>>,
}

#[derive(Debug, Deserialize)]
struct SettingsSection {
    dry_run: Option<bool>,
    verbose: Option<bool>,
    clear_terminal: Option<bool>,
}

#[derive(Debug, Deserialize)]
struct AccessSection {
    lastseen: Option<u64>,
}

impl Default for Config {
    fn default() -> Self {
        Self {
            search_paths: vec![dirs::home_dir().unwrap_or_else(|| PathBuf::from("."))],
            exclude_patterns: vec![
                ".git".to_string(),
                "node_modules".to_string(),
                ".vscode".to_string(),
                ".cargo".to_string(),
                ".rustup".to_string(),
            ],
            ignore_paths: Vec::new(),
            stale_threshold: Duration::from_secs(7 * 24 * 60 * 60), // 7 days
            last_access_days: 7, // Default to 7 days for last access check
            dry_run: true,
            verbose: false,
            clear_terminal: true, // Default to clearing terminal before UI
        }
    }
}

impl Config {
    pub fn new() -> Self {
        Self::default()
    }

    #[allow(dead_code)]
    pub fn with_search_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.search_paths = paths;
        self
    }

    #[allow(dead_code)]
    pub fn with_exclude_patterns(mut self, patterns: Vec<String>) -> Self {
        self.exclude_patterns = patterns;
        self
    }

    #[allow(dead_code)]
    pub fn with_stale_threshold(mut self, threshold: Duration) -> Self {
        self.stale_threshold = threshold;
        self
    }

    #[allow(dead_code)]
    pub fn with_dry_run(mut self, dry_run: bool) -> Self {
        self.dry_run = dry_run;
        self
    }

    #[allow(dead_code)]
    pub fn with_verbose(mut self, verbose: bool) -> Self {
        self.verbose = verbose;
        self
    }

    #[allow(dead_code)]
    pub fn with_ignore_paths(mut self, paths: Vec<PathBuf>) -> Self {
        self.ignore_paths = paths;
        self
    }
    #[allow(dead_code)]
    /// Sets whether to clear the terminal before starting the UI
    pub fn with_clear(mut self, clear: bool) -> Self {
        self.clear_terminal = clear;
        self
    }

    /// Load configuration from a Cleaner.toml file using proper TOML deserialization
    pub fn load_cleaner_config(
        &mut self,
        config_path: &Path,
    ) -> Result<(), Box<dyn std::error::Error>> {
        if !config_path.exists() {
            return Ok(()); // It's okay if the file doesn't exist
        }

        let content = fs::read_to_string(config_path)?;
        let config: CleanerConfig = toml::from_str(&content)?;
        println!("CleanerConfig {:?}", config);
        // Process ignore paths
        if let Some(ignore) = config.ignore {
            if let Some(paths) = ignore.paths {
                for path_str in paths {
                    let path = PathBuf::from(path_str);
                    // Add path regardless of whether it exists now
                    // The scanner will handle non-existent paths gracefully
                    self.ignore_paths.push(path);
                }
            }
        }

        // Process settings
        if let Some(settings) = config.settings {
            if let Some(dry_run) = settings.dry_run {
                self.dry_run = dry_run;
            }
            if let Some(verbose) = settings.verbose {
                self.verbose = verbose;
            }
            if let Some(clear_terminal) = settings.clear_terminal {
                self.clear_terminal = clear_terminal;
            }
        }

        // Process access settings
        if let Some(access) = config.access {
            if let Some(lastseen) = access.lastseen {
                self.last_access_days = lastseen;
                self.stale_threshold = Duration::from_secs(lastseen * 24 * 60 * 60);
            }
        }

        Ok(())
    }
}
