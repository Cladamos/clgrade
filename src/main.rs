mod app;
mod image;
mod input;
mod preset;
mod ui;

use std::io::stdout;

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

    let mut app = App::new();
    let app_result = app.run(&mut terminal);
    execute!(stdout(), PopKeyboardEnhancementFlags,)?;
    ratatui::restore();
    crossterm::terminal::disable_raw_mode()?;

    app_result.map_err(|e| color_eyre::eyre::eyre!("Application failed: {}", e))
}
