# Rust Target Cleaner

A utility tool to identify and remove stale target directories from Rust projects that haven't been used in a configurable time period (default: 7 days). This helps free up disk space by cleaning up build artifacts that are no longer needed.

## Features

- Scans for Rust projects in specified directories
- Identifies target directories based on modification time
- Interactive terminal UI with dancing Rust crab animation during scanning
- Review and select targets for cleanup
- Configurable age threshold for stale targets
- Dry-run mode to preview what would be deleted without actually deleting
- Reports space that would be freed
- Configuration file support for ignore paths and settings
- Cross-platform support (Linux, macOS, Windows)

## Installation

```bash
git clone https://github.com/yourusername/rust_clear_target.git
cd rust_clear_target
cargo build --release
```

## Usage

Run the application:

```bash
cargo run
```

Or if you've built it with `--release`:

```bash
./target/release/rust_clear_target
```

### Terminal UI Controls

- **Arrow Keys** (↑/↓): Navigate through the project list
- **Space**: Select/deselect a project for cleanup
- **Enter**: Confirm selection and start cleanup process
- **Y/N**: Confirm or cancel the deletion in confirmation mode
- **Q**: Quit the application

### Color Indicators

- 🟢 Green: Target directory is actively used (modified recently)
- 🔴 Red: Target directory is stale (not modified within threshold)

## Configuration

You can customize the behavior of the tool by creating a `Cleaner.toml` file in your home directory.

### Example Cleaner.toml

```toml
[ignore.paths]
# Add directories to ignore during scanning
path = "/home/user/important_project"
path = "/opt/rust/projects/production"
/home/user/dont_scan_this

[settings]
# How old (in days) a target directory must be before it's considered stale
stale_threshold_days = 7

# Whether to run in dry-run mode by default
dry_run = true
```

### Configuration Options

#### [ignore.paths] Section
List directories you want to exclude from scanning. You can specify paths in two formats:
1. With explicit key: `path = "/full/path/to/directory"`
2. Direct listing: `/full/path/to/directory`

#### [settings] Section
- `stale_threshold_days`: Number of days before a target directory is considered stale (default: 7)
- `dry_run`: Whether to run in dry-run mode by default (default: true)
- `verbose`: Whether to show verbose output (default: false)

## Configuration

The tool currently uses default configuration, but you can customize the behavior by modifying the source code:

- `stale_threshold`: How old a target directory must be before it's considered stale (default: 30 days)
- `search_paths`: Directories to scan for Rust projects (default: home directory)
- `exclude_patterns`: Patterns to exclude from scanning (default: `.git`, `node_modules`, `.vscode`)
- `dry_run`: Run in preview mode without actual deletion (default: `true`)

## How It Works

1. The scanner walks through your configured search paths looking for `Cargo.toml` files
2. For each Rust project found, it checks for a `target` directory
3. It analyzes the target directory's modification time to determine if it's stale
4. During scanning, a dancing Rust crab animation shows progress
5. The interactive UI displays projects with size and age information
6. You can select which target directories to clean up
6. The tool removes the selected directories and reports the space freed

## Safety

- The tool starts in dry-run mode by default, showing what would be deleted without actually deleting
- It checks for project activity based on the last access time of the target directory
- You must explicitly confirm before any deletions occur
- Target directories are only deleted if they're not actively used

## Dependencies

- `chrono`: For date/time calculations
- `crossterm`: For terminal handling
- `dirs`: For finding user home directories
- `ratatui`: For building the terminal user interface
- `walkdir`: For efficiently traversing directory structures

## Future Enhancements

- Command-line arguments for configuration
- Configuration file support
- Scheduled cleanup via daemon/cron
- Custom exclusion rules
- Integration with package managers
# rust_clear_target
