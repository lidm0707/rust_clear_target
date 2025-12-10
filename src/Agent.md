# Rust Target Cleaner Agent

## Project Overview
The Rust Target Cleaner is a utility tool designed to identify and optionally remove target directories from Rust projects that haven't been accessed within a configurable time period (default: 30 days). This helps free up disk space by cleaning up build artifacts that are no longer needed.

## Key Features
1. Scan for Rust projects in specified directories
2. Check last access time of target directories
3. Interactive UI for reviewing and selecting targets for cleanup
4. Configurable age threshold for stale targets
5. Dry-run mode to preview what would be deleted
6. Statistics reporting (space freed, targets processed)
7. Command-line interface with terminal UI

## Project Structure
```
rust_clear_target/
├── Cargo.toml
├── src/
│   ├── main.rs           # Application entry point
│   ├── agent.md          # This file - project plan
│   ├── app.rs            # Main application logic
│   ├── ui/
│   │   ├── mod.rs        # UI module
│   │   ├── tui.rs        # Terminal UI implementation
│   │   └── components.rs # UI components
│   ├── scanner/
│   │   ├── mod.rs        # Scanner module
│   │   ├── rust_project.rs # Rust project detection
│   │   └── target_finder.rs # Target directory discovery
│   ├── cleaner/
│   │   ├── mod.rs        # Cleaner module
│   │   ├── analyzer.rs   # Analysis of target directories
│   │   └── remover.rs    # Safe removal implementation
│   └── config.rs         # Configuration management
└── README.md
```

## Implementation Plan

### Phase 1: Core Structure and Project Detection
1. Set up basic application structure
2. Implement Rust project detection logic
   - Look for Cargo.toml files
   - Verify associated target directories
3. Create target directory scanner
   - Use walkdir to traverse file systems
   - Collect metadata (size, access times)

### Phase 2: Analysis Engine
1. Implement target directory analysis
   - Calculate age based on last access time
   - Determine size of target directories
   - Check for locked files or processes using targets
2. Create filtering logic based on age threshold
3. Add dry-run capabilities

### Phase 3: Terminal UI
1. Implement TUI using ratatui
   - Project list with details (size, age, path)
   - Selection interface for targets to clean
   - Confirmation prompts
   - Progress indicators
2. Add interactive features
   - Toggle selection of projects
   - Filter by size or age
   - Sort options

### Phase 4: Cleanup Operations
1. Implement safe removal logic
   - Verify targets aren't in use
   - Handle permission issues gracefully
   - Provide detailed error reporting
2. Add rollback capability (logging what was deleted)

### Phase 5: Configuration and CLI
1. Implement configuration management
   - Default age threshold (30 days)
   - Custom scan paths
   - Exclusion patterns
2. Add command-line argument parsing
   - Options for different operation modes
   - Verbose output settings

## Technical Implementation Details

### Dependencies Utilization
- **chrono**: For date/time calculations and comparisons
- **crossterm**: For terminal handling and cross-platform support
- **dirs**: For finding user home directories and default search paths
- **ratatui**: For building the terminal user interface
- **walkdir**: For efficiently traversing directory structures

### Core Components

#### Project Detection
```rust
// Scanner for finding Rust projects
pub struct RustProjectScanner {
    search_paths: Vec<PathBuf>,
    exclude_patterns: Vec<String>,
}

impl RustProjectScanner {
    pub fn find_projects(&self) -> Result<Vec<RustProject>, ScanError>;
    pub fn has_target_directory(&self, project: &Path) -> bool;
}
```

#### Target Analysis
```rust
// Analyzer for target directories
pub struct TargetAnalyzer {
    stale_threshold: Duration,
}

impl TargetAnalyzer {
    pub fn is_stale(&self, target_path: &Path) -> Result<bool, AnalysisError>;
    pub fn calculate_size(&self, target_path: &Path) -> Result<u64, AnalysisError>;
}
```

#### UI Implementation
```rust
// Terminal UI for interaction
pub struct CleanerTUI {
    app: App,
    terminal: Terminal<CrosstermBackend<Stdout>>,
}

impl CleanerTUI {
    pub fn run(&mut self) -> Result<(), io::Error>;
    // Methods for handling events and rendering UI
}
```

### Error Handling Strategy
- Use custom error types for different modules
- Implement graceful degradation for non-critical errors
- Provide user-friendly error messages
- Log detailed errors for debugging

### Testing Strategy
1. Unit tests for individual components
2. Integration tests for core workflows
3. Mock file system for test scenarios
4. CLI tests for command-line interface

## Future Enhancements
1. Scheduled cleanup via daemon/cron
2. Integration with package managers (cargo, rustup)
3. Web UI for remote management
4. Configuration persistence across sessions
5. Custom cleaning rules based on project patterns
6. Integration with CI/CD pipelines

## Release Plan
1. v0.1.0: Basic functionality with CLI
2. v0.2.0: Interactive TUI implementation
3. v0.3.0: Configuration and customization options
4. v1.0.0: Full feature set with documentation