use std::error::Error;

mod app;
mod cleaner;
mod config;
mod scanner;
mod ui;
use app::App;
use config::Config;

fn main() -> Result<(), Box<dyn Error>> {
    // toml config not working
    let config = Config::new();
    println!("{:?}", config);
    let mut app = App::new(config)?;

    app.run()?;

    Ok(())
}
