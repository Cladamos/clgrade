mod app;
mod image;
mod ui;

use app::App;
use color_eyre::Result;

fn main() -> Result<()> {
    color_eyre::install()?;

    let mut app = App::new();
    ratatui::run(|terminal| app.run(terminal))?;
    Ok(())
}
