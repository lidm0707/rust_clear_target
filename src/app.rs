use crate::config::Config;
use crate::scanner::rust_project_scaner::RustProjectScanner;
use crate::ui::{CleanerTUI, UI};
use std::error::Error;

pub struct App {
    config: Config,
    scanner: RustProjectScanner,
}

impl App {
    pub fn new(mut config: Config) -> Result<Self, Box<dyn Error>> {
        // Load configuration from Cleaner.toml if it exists
        let dir = std::env::current_dir()?;

        // let config_path = dirs::home_dir()
        //     .unwrap_or_else(|| PathBuf::from("."))
        //     .join("Cleaner.toml");
        let config_path = dir.join("Cleaner.toml");

        println!("current {:?}", config_path);
        if let Err(e) = config.load_cleaner_config(&config_path) {
            eprintln!("Warning: Failed to load Cleaner.toml: {}", e);
        }

        println!("Config pass {:?}", config);

        let scanner = RustProjectScanner::new_with_ignores(
            &config.search_paths,
            &config.exclude_patterns,
            &config.ignore_paths,
        )?;

        Ok(App { config, scanner })
    }

    pub fn run(&mut self) -> Result<(), Box<dyn Error>> {
        use std::io::{Write, stdout};
        use std::thread;
        use std::time::Duration;

        // Print header once
        println!("Scanning for Rust projects...");

        // (1) setup animation thread
        let (tx, rx) = std::sync::mpsc::channel();

        let loading_indicator = thread::spawn(move || {
            // let crab_frames = [
            //     "🦀 ▽   ",
            //     "~ ▽   ",
            //     "  ▽   ",
            //     "  ▽ ~ ",
            //     "  ▽   ",
            //     "▽ ~   ",
            //     "  ▽   ",
            // ];
            let crab_frames = [
                "   ╲╱   🦀  ▽     / ╲  ",
                "   ╲╱   🦀  ~     / ╲  ",
                "   ╲╱   🦀       / ╲  ",
                "   ╲╱   🦀      / ╲  ",
                "   ╲╱   🦀     / ╲  ",
                "   ╲╱   🦀    / ╲  ",
                "   ╲╱   🦀   / ╲  ",
                "   ╲╱   ▽ ~      / ╲  ",
                "   ╲╱     ▽     / ╲  ",
            ];
            let mut i = 0;
            while rx.try_recv().is_err() {
                print!("\rScanning... {}", crab_frames[i]);
                stdout().flush().unwrap();

                i = (i + 1) % crab_frames.len();
                thread::sleep(Duration::from_millis(120));
            }

            // After stop → clear animation
            print!("\rScanning complete!     \n");
            stdout().flush().unwrap();
        });

        // (2) do your scanning
        let projects = self.scanner.find_projects()?;

        // (3) stop animation
        tx.send(()).ok();
        loading_indicator.join().ok();

        println!(
            "Found {} Rust projects with target directories",
            projects.len()
        );

        // (4) start ratatui
        let mut tui = CleanerTUI::new(projects, self.config.clone())?;
        tui.run()?;

        Ok(())
    }
}
