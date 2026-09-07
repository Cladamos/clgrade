mod app;
mod image;
mod input;
mod preset;
mod ui;

use std::io::stdout;
use std::path::PathBuf;

use app::App;
use color_eyre::Result;
use crossterm::{
    event::{KeyboardEnhancementFlags, PopKeyboardEnhancementFlags, PushKeyboardEnhancementFlags},
    execute,
};
use ratatui::crossterm;

fn main() -> Result<()> {
    color_eyre::install()?;
    let mut terminal = ratatui::init();

    crossterm::terminal::enable_raw_mode()?;
    execute!(
        stdout(),
        PushKeyboardEnhancementFlags(KeyboardEnhancementFlags::REPORT_EVENT_TYPES)
    )?;

    let initial_image = parse_cli_args();
    let mut app = App::new(initial_image);
    let app_result = app.run(&mut terminal);
    execute!(stdout(), PopKeyboardEnhancementFlags,)?;
    ratatui::restore();
    crossterm::terminal::disable_raw_mode()?;

    app_result.map_err(|e| color_eyre::eyre::eyre!("Application failed: {}", e))
}

pub const SUPPORTED_FORMATS: &[&str] = &["png", "jpg", "jpeg", "webp"];

fn parse_cli_args() -> Option<PathBuf> {
    let args: Vec<String> = std::env::args().collect();

    if args.len() < 2 {
        return None;
    }

    if args[1] == "--help" || args[1] == "-h" {
        ratatui::restore();
        eprintln!("Usage: {} [IMAGE_PATH]", args[0]);
        eprintln!();
        eprintln!("  IMAGE_PATH  Optional path to an image file (png, jpg, jpeg, webp)");
        eprintln!();
        eprintln!("  If no path is given, image can be selected using the file explorer");
        std::process::exit(0);
    }

    let path = PathBuf::from(&args[1]);

    if !path.exists() {
        ratatui::restore();
        eprintln!("Error: File not found: {}", args[1]);
        std::process::exit(1);
    }

    let ext = path
        .extension()
        .and_then(|e| e.to_str())
        .unwrap_or("")
        .to_lowercase();

    if !SUPPORTED_FORMATS.contains(&ext.as_str()) {
        ratatui::restore();
        eprintln!(
            "Error: Unsupported format '{}'. Supported: {}",
            ext,
            SUPPORTED_FORMATS.join(", ")
        );
        std::process::exit(1);
    }
    Some(std::fs::canonicalize(&path).unwrap_or(path))
}
