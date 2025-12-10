use std::error::Error;

mod tui;

pub use tui::CleanerTUI;

/// Common UI trait for different UI implementations
pub trait UI {
    /// Runs the UI and returns the result
    fn run(&mut self) -> Result<(), Box<dyn Error>>;
}
