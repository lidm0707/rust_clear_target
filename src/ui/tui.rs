use std::error::Error;
use std::io::{self, Stdout, Write};
use std::time::{Duration, SystemTime};

use crossterm::event::{KeyEvent, KeyModifiers};
use crossterm::{
    event::{self, DisableMouseCapture, EnableMouseCapture, Event, KeyCode},
    execute,
    terminal::{EnterAlternateScreen, LeaveAlternateScreen, disable_raw_mode, enable_raw_mode},
};
use ratatui::{
    Frame, Terminal,
    backend::CrosstermBackend,
    layout::{Constraint, Direction, Layout, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, Borders, Gauge, List, ListItem, ListState, Paragraph, Wrap},
};

use crate::cleaner::targer_cleaner::TargetCleaner;
use crate::config::Config;
use crate::scanner::rust_project::RustProject;
use crate::scanner::target_finder::TargetFinder;
use crate::ui::UI;

/// Terminal UI for the Rust target cleaner
pub struct CleanerTUI {
    /// List of Rust projects found
    projects: Vec<RustProject>,
    /// Application configuration
    config: Config,
    /// Terminal interface
    terminal: Terminal<CrosstermBackend<Stdout>>,
    /// Current state of the application
    state: AppState,
}

/// Application state
#[derive(Debug)]
pub struct AppState {
    /// Currently selected item in the list
    selected: usize,
    /// List state for ratatui
    list_state: ListState,
    /// Which projects are selected for cleaning
    selected_projects: Vec<bool>,
    /// Current UI mode
    mode: UIMode,
    /// Status message to display
    status_message: String,
    /// Total space that would be freed
    total_freed_space: u64,
    /// Progress for cleanup operation
    cleanup_progress: f32,
}

/// UI modes
#[derive(Debug, PartialEq, Eq)]
pub enum UIMode {
    /// Normal browsing mode
    Browse,
    /// Selection confirmation mode
    Confirm,
    /// Cleaning in progress
    Cleaning,
    /// Cleanup complete
    Complete,
}

impl UI for CleanerTUI {
    fn run(&mut self) -> Result<(), Box<dyn Error>> {
        self.run_internal()
    }
}

impl CleanerTUI {
    /// Creates a new terminal UI instance
    pub fn new(projects: Vec<RustProject>, config: Config) -> Result<Self, Box<dyn Error>> {
        // Clear terminal if configured
        if config.clear_terminal {
            print!("\x1B[2J\x1B[H");
            std::io::stdout().flush()?;
        }

        // Initialize terminal
        enable_raw_mode()?;
        let mut stdout = io::stdout();
        execute!(stdout, EnterAlternateScreen, EnableMouseCapture)?;
        let backend = CrosstermBackend::new(stdout);
        let terminal = Terminal::new(backend)?;

        // Update target info with stale status
        let mut updated_projects = Vec::new();
        for project in projects {
            if let Some(target_info) = &project.target_info {
                let mut target_info_clone = target_info.clone();
                TargetFinder::update_stale_status(&mut target_info_clone, config.stale_threshold)?;
                let project_with_updated_target =
                    project.clone().with_target_info(target_info_clone);
                updated_projects.push(project_with_updated_target);
            } else {
                updated_projects.push(project.clone());
            }
        }

        // Initialize application state
        let selected_projects = vec![false; updated_projects.len()];
        let mut list_state = ListState::default();
        list_state.select(Some(0));

        let state = AppState {
            selected: 0,
            list_state,
            selected_projects,
            mode: UIMode::Browse,
            status_message:
                "Use arrow keys to navigate, Space to select, Enter to confirm, 'q' to quit"
                    .to_string(),
            total_freed_space: 0,
            cleanup_progress: 0.0,
        };

        Ok(Self {
            projects: updated_projects,
            config,
            terminal,
            state,
        })
    }

    /// Runs the terminal UI
    fn run_internal(&mut self) -> Result<(), Box<dyn Error>> {
        loop {
            // Draw the UI
            {
                let state = &self.state;
                let projects = &self.projects;
                let config = &self.config;
                let total_freed_space = self.state.total_freed_space;
                let status_message = &self.state.status_message;

                self.terminal.draw(|f| {
                    Self::draw_ui_static(
                        f,
                        state,
                        projects,
                        config,
                        total_freed_space,
                        status_message,
                    );
                })?;
            }

            // Handle events
            if let Event::Key(key) = event::read()? {
                match self.state.mode {
                    UIMode::Browse => self.handle_browse_mode(key)?,
                    UIMode::Confirm => self.handle_confirm_mode(key)?,
                    UIMode::Cleaning => self.handle_cleaning_mode(key)?,
                    UIMode::Complete => self.handle_complete_mode(key)?,
                }
            }

            // Check if we should exit
            if self.should_exit() {
                break;
            }
        }

        // Restore terminal
        self.restore_terminal()?;
        Ok(())
    }

    /// Handles key events in browse mode
    fn handle_browse_mode(&mut self, key: event::KeyEvent) -> Result<(), Box<dyn Error>> {
        match key {
            KeyEvent {
                code: KeyCode::Up, ..
            } => {
                if self.state.selected > 0 {
                    self.state.selected -= 1;
                    self.state.list_state.select(Some(self.state.selected));
                }
            }
            KeyEvent {
                code: KeyCode::Down,
                ..
            } => {
                if self.state.selected < self.projects.len().saturating_sub(1) {
                    self.state.selected += 1;
                    self.state.list_state.select(Some(self.state.selected));
                }
            }
            KeyEvent {
                code: KeyCode::Char(' '),
                ..
            } => {
                if !self.projects.is_empty() {
                    self.state.selected_projects[self.state.selected] =
                        !self.state.selected_projects[self.state.selected];
                    self.update_total_freed_space();
                }
            }
            KeyEvent {
                code: KeyCode::Enter,
                ..
            } => {
                let selected_count = self.state.selected_projects.iter().filter(|&x| *x).count();
                if selected_count > 0 {
                    self.state.mode = UIMode::Confirm;
                    self.state.status_message = format!(
                        "Confirm deletion of {} target directories? (y/N)",
                        selected_count
                    );
                } else {
                    self.state.status_message =
                        "No projects selected. Use Space to select projects.".to_string();
                }
            }

            KeyEvent {
                code: KeyCode::Char('c'),
                modifiers: KeyModifiers::CONTROL,
                ..
            } => {
                self.state.mode = UIMode::Complete;
                return Ok(());
            }

            _ => {}
        }
        Ok(())
    }

    /// Handles key events in confirmation mode
    fn handle_confirm_mode(&mut self, key: event::KeyEvent) -> Result<(), Box<dyn Error>> {
        match key.code {
            KeyCode::Char('y') | KeyCode::Char('Y') => {
                self.state.mode = UIMode::Cleaning;
                self.state.status_message = "Cleaning target directories...".to_string();
                self.perform_cleanup()?;
            }
            KeyCode::Char('n') | KeyCode::Char('N') | KeyCode::Esc => {
                self.state.mode = UIMode::Browse;
                self.state.status_message = "Operation cancelled. Use arrow keys to navigate, Space to select, Enter to confirm, 'q' to quit".to_string();
            }
            _ => {}
        }
        Ok(())
    }

    /// Handles key events in cleaning mode
    fn handle_cleaning_mode(&mut self, _key: event::KeyEvent) -> Result<(), Box<dyn Error>> {
        // In cleaning mode, input is disabled
        Ok(())
    }

    /// Handles key events in complete mode
    fn handle_complete_mode(&mut self, key: event::KeyEvent) -> Result<(), Box<dyn Error>> {
        match key.code {
            KeyCode::Char('d') => {
                // Toggle between dry run and live mode
                self.config.dry_run = !self.config.dry_run;
                let mode = if self.config.dry_run {
                    "dry run"
                } else {
                    "live"
                };
                self.state.status_message = format!("Switched to {} mode", mode);
            }
            KeyCode::Char('r') => {
                // Reset to selection mode to choose different projects
                self.state.mode = UIMode::Browse;
                self.state.status_message =
                    "Back to selection mode. Use arrow keys to navigate, Space to select, Enter to confirm, 'q' to quit"
                    .to_string();
            }
            KeyCode::Enter | KeyCode::Char('q') => {
                return Ok(());
            }
            _ => {}
        }
        Ok(())
    }

    /// Performs the cleanup operation
    fn perform_cleanup(&mut self) -> Result<(), Box<dyn Error>> {
        let total_to_clean = self.state.selected_projects.iter().filter(|&x| *x).count();
        let mut cleaned = 0;

        for (i, project) in self.projects.iter().enumerate() {
            if self.state.selected_projects[i] {
                if project.target_info.is_some() {
                    // Simulate cleanup progress
                    cleaned += 1;
                    self.state.cleanup_progress = cleaned as f32 / total_to_clean as f32;

                    // Redraw to update progress
                    {
                        let state = &self.state;
                        let projects = &self.projects;
                        let config = &self.config;
                        let total_freed_space = self.state.total_freed_space;
                        let status_message = &self.state.status_message;

                        self.terminal.draw(|f| {
                            Self::draw_ui_static(
                                f,
                                state,
                                projects,
                                config,
                                total_freed_space,
                                status_message,
                            );
                        })?;
                    }

                    // Use our TargetCleaner to perform the cleanup
                    match TargetCleaner::clean_selected_projects(
                        &self.projects,
                        &self.state.selected_projects,
                        self.config.dry_run,
                    ) {
                        Ok(result) => {
                            if self.config.dry_run {
                                self.state.status_message = format!(
                                    "Dry run complete. Would have freed {} of space.",
                                    format_bytes(result.total_freed)
                                );
                            } else {
                                self.state.status_message = format!(
                                    "Cleanup complete. Freed {} of space. {} errors occurred.",
                                    format_bytes(result.total_freed),
                                    result.errors.len()
                                );

                                // Show errors if any occurred
                                for error in &result.errors {
                                    eprintln!("Error: {}", error);
                                }
                            }
                            self.state.total_freed_space = result.total_freed;
                        }
                        Err(e) => {
                            self.state.status_message = format!("Error during cleanup: {}", e);
                        }
                    }
                }
            }
        }

        // Transition to complete mode
        self.state.mode = UIMode::Complete;

        if self.config.dry_run {
            self.state.status_message = format!(
                "Dry run complete. Would have freed {} of space. Press Enter or q to exit.",
                format_bytes(self.state.total_freed_space)
            );
        } else {
            self.state.status_message = format!(
                "Cleanup complete. Freed {} of space. Press Enter or q to exit.",
                format_bytes(self.state.total_freed_space)
            );
        }

        self.state.cleanup_progress = 1.0;

        Ok(())
    }

    /// Updates the total space that would be freed
    fn update_total_freed_space(&mut self) {
        self.state.total_freed_space = 0;
        for (i, project) in self.projects.iter().enumerate() {
            if self.state.selected_projects[i] {
                if let Some(ref target_info) = project.target_info {
                    self.state.total_freed_space += target_info.size_bytes;
                }
            }
        }
    }

    /// Draws the UI
    #[allow(dead_code)]
    fn draw_ui(&mut self, f: &mut Frame) {
        Self::draw_ui_static(
            f,
            &self.state,
            &self.projects,
            &self.config,
            self.state.total_freed_space,
            &self.state.status_message,
        );
    }

    /// Static method to draw the UI without borrowing issues
    fn draw_ui_static(
        f: &mut Frame,
        state: &AppState,
        projects: &[RustProject],
        config: &Config,
        total_freed_space: u64,
        status_message: &str,
    ) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Min(10),   // Main content
                Constraint::Length(3), // Status bar
            ])
            .split(f.area());

        // Draw main content
        match state.mode {
            UIMode::Browse | UIMode::Confirm => {
                Self::draw_project_list_static(f, chunks[0], state, projects)
            }
            UIMode::Cleaning => Self::draw_progress_static(f, chunks[0], state, status_message),
            UIMode::Complete => {
                Self::draw_complete_static(f, chunks[0], config, total_freed_space, status_message)
            }
        }

        // Draw status bar
        Self::draw_status_bar_static(
            f,
            chunks[1],
            state,
            projects.len(),
            config,
            total_freed_space,
            status_message,
        );
    }

    /// Draws the project list
    #[allow(dead_code)]
    fn draw_project_list(&mut self, f: &mut Frame, area: Rect) {
        Self::draw_project_list_static(f, area, &self.state, &self.projects);
    }

    /// Static method to draw the project list without borrowing issues
    fn draw_project_list_static(
        f: &mut Frame,
        area: Rect,
        state: &AppState,
        projects: &[RustProject],
    ) {
        // Create list items from projects
        let items: Vec<ListItem> = projects
            .iter()
            .enumerate()
            .map(|(i, project)| {
                let (name, path, size, age) = if let Some(ref target_info) = project.target_info {
                    let is_stale = target_info.is_stale;
                    let duration_since = SystemTime::now()
                        .duration_since(target_info.last_accessed)
                        .unwrap_or_else(|_| Duration::from_secs(30 * 24 * 60 * 60));

                    let age_display = if duration_since.as_secs() < 86400 {
                        "Today".to_string()
                    } else if duration_since.as_secs() < 2 * 86400 {
                        "Yesterday".to_string()
                    } else {
                        let days = duration_since.as_secs() / 86400;
                        if days < 30 {
                            format!("{} days ago", days)
                        } else if days < 365 {
                            format!("{} months ago", days / 30)
                        } else {
                            format!("{} years ago", days / 365)
                        }
                    };

                    let status_indicator = if is_stale { "🔴" } else { "🟢" };

                    (
                        format!("{} {}", status_indicator, project.name),
                        format!("{}", project.path.display()),
                        format!("{}", format_bytes(target_info.size_bytes)),
                        age_display,
                    )
                } else {
                    (
                        format!("🔴 {}", project.name),
                        format!("{}", project.path.display()),
                        "No target".to_string(),
                        "N/A".to_string(),
                    )
                };

                let is_selected = state.selected_projects.get(i).copied().unwrap_or(false);
                let line_color = if is_selected {
                    Color::Yellow
                } else {
                    Color::White
                };
                let line_style = Style::default().fg(line_color);

                let content = vec![
                    Line::from(Span::styled(name, line_style.add_modifier(Modifier::BOLD))),
                    Line::from(Span::styled(path, line_style)),
                    Line::from(vec![
                        Span::styled("Size: ", Style::default()),
                        Span::styled(size, line_style.add_modifier(Modifier::DIM)),
                        Span::raw("  "),
                        Span::styled("Last accessed: ", Style::default()),
                        Span::styled(age, line_style.add_modifier(Modifier::DIM)),
                    ]),
                ];

                ListItem::new(content)
            })
            .collect();

        // Create the list widget
        let list = List::new(items)
            .block(
                Block::default()
                    .borders(Borders::ALL)
                    .title("Rust Projects"),
            )
            .highlight_style(
                Style::default()
                    .bg(Color::DarkGray)
                    .add_modifier(Modifier::BOLD),
            );

        // Render the list
        let mut list_state = state.list_state.clone();
        f.render_stateful_widget(list, area, &mut list_state);
    }

    /// Draws the progress view during cleanup
    #[allow(dead_code)]
    fn draw_progress(&mut self, f: &mut Frame, area: Rect) {
        Self::draw_progress_static(f, area, &self.state, &self.state.status_message);
    }

    /// Static method to draw the progress view without borrowing issues
    fn draw_progress_static(f: &mut Frame, area: Rect, state: &AppState, status_message: &str) {
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(3),  // Title
                Constraint::Length(10), // Progress bar
                Constraint::Min(1),     // Status
            ])
            .split(area);

        // Title
        let title = Paragraph::new("Cleaning target directories...")
            .block(Block::default().borders(Borders::ALL))
            .style(
                Style::default()
                    .fg(Color::Cyan)
                    .add_modifier(Modifier::BOLD),
            );
        f.render_widget(title, chunks[0]);

        // Progress bar
        let gauge = Gauge::default()
            .block(Block::default().borders(Borders::ALL).title("Progress"))
            .gauge_style(Style::default().fg(Color::Green).bg(Color::Black))
            .percent((state.cleanup_progress * 100.0) as u16);
        f.render_widget(gauge, chunks[1]);

        // Status
        let status = Paragraph::new(status_message)
            .block(Block::default().borders(Borders::ALL))
            .wrap(Wrap { trim: true });
        f.render_widget(status, chunks[2]);
    }

    /// Draws the completion view
    #[allow(dead_code)]
    fn draw_complete(&mut self, f: &mut Frame, area: Rect) {
        Self::draw_complete_static(
            f,
            area,
            &self.config,
            self.state.total_freed_space,
            &self.state.status_message,
        );
    }

    /// Static method to draw the completion view without borrowing issues
    fn draw_complete_static(
        f: &mut Frame,
        area: Rect,
        config: &Config,
        total_freed_space: u64,
        status_message: &str,
    ) {
        let text = if config.dry_run {
            format!(
                "Dry run completed! Would have freed {} of space.\n\n{}",
                format_bytes(total_freed_space),
                status_message
            )
        } else {
            format!(
                "Cleanup completed successfully! Freed {} of space.\n\n{}",
                format_bytes(total_freed_space),
                status_message
            )
        };

        let paragraph = Paragraph::new(text)
            .block(Block::default().borders(Borders::ALL).title("Complete"))
            .style(
                Style::default()
                    .fg(Color::Green)
                    .add_modifier(Modifier::BOLD),
            )
            .wrap(Wrap { trim: true });
        f.render_widget(paragraph, area);
    }

    /// Draws the status bar
    #[allow(dead_code)]
    fn draw_status_bar(&mut self, f: &mut Frame, area: Rect) {
        Self::draw_status_bar_static(
            f,
            area,
            &self.state,
            self.projects.len(),
            &self.config,
            self.state.total_freed_space,
            &self.state.status_message,
        );
    }

    /// Static method to draw the status bar without borrowing issues
    fn draw_status_bar_static(
        f: &mut Frame,
        area: Rect,
        state: &AppState,
        project_count: usize,
        config: &Config,
        total_freed_space: u64,
        status_message: &str,
    ) {
        let selected_count = state.selected_projects.iter().filter(|&x| *x).count();
        let status_text = format!(
            "{} | Selected: {}/{} | Space to free: {} | {}",
            if config.dry_run {
                "Dry Run (press 'd' to toggle live mode)"
            } else {
                "Live Mode (press 'd' to toggle dry run)"
            },
            selected_count,
            project_count,
            format_bytes(total_freed_space),
            status_message
        );

        let status_bar =
            Paragraph::new(status_text).style(Style::default().bg(Color::Blue).fg(Color::White));
        f.render_widget(status_bar, area);
    }

    /// Checks if we should exit the application
    fn should_exit(&self) -> bool {
        matches!(self.state.mode, UIMode::Complete)
    }

    /// Clears the terminal if configured to do so
    #[allow(dead_code)]
    fn clear_terminal_if_needed(&mut self) -> Result<(), Box<dyn Error>> {
        if self.config.clear_terminal {
            print!("\x1B[2J\x1B[H");
            std::io::stdout().flush()?;
        }
        Ok(())
    }

    /// Restores the terminal state
    fn restore_terminal(&mut self) -> Result<(), Box<dyn Error>> {
        disable_raw_mode()?;
        execute!(
            self.terminal.backend_mut(),
            LeaveAlternateScreen,
            DisableMouseCapture
        )?;
        self.terminal.show_cursor()?;
        Ok(())
    }
}

/// Formats bytes into a human-readable string
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
