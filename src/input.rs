use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::crossterm;

pub enum Action {
    Quit,
    ExportImage,
    ToggleFileExplorer,
    NextTool,
    IncreaseValue,
    IncreaseValueSideways,
    DecreaseValue,
    DecreaseValueSideways,
    ResetTool,
    SwitchToSliders,
    SwitchToWheels,
    ChangeAspectRatio,
    ChangeResolution,
    ResetAll,
    Select,
    Save,
    ToggleOriginal,
    ToggleProxy,
    None,
}

pub fn map_key_to_action(key: KeyEvent) -> Action {
    match (key.modifiers, key.code) {
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => Action::ExportImage,
        (_, KeyCode::Char('q')) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => Action::Quit,
        (_, KeyCode::Enter) => Action::Select,
        (_, KeyCode::Char('s')) => Action::Save,
        (_, KeyCode::Char('f')) => Action::ToggleFileExplorer,
        (_, KeyCode::Char('p')) => Action::ToggleProxy,
        (_, KeyCode::Char(' ')) => Action::ToggleOriginal,

        // Navigation
        (_, KeyCode::Tab) => Action::NextTool,
        (_, KeyCode::Char('1')) => Action::SwitchToSliders,
        (_, KeyCode::Char('2')) => Action::SwitchToWheels,

        // Manipulation
        (_, KeyCode::Up) | (_, KeyCode::Char('k')) => Action::IncreaseValue,
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => Action::DecreaseValue,
        (_, KeyCode::Right) | (_, KeyCode::Char('l')) => Action::IncreaseValueSideways,
        (_, KeyCode::Left) | (_, KeyCode::Char('h')) => Action::DecreaseValueSideways,
        (_, KeyCode::Char('a')) => Action::ChangeAspectRatio,
        (_, KeyCode::Char('A')) => Action::ChangeResolution,
        (_, KeyCode::Char('r')) => Action::ResetTool,
        (_, KeyCode::Char('R')) => Action::ResetAll,

        _ => Action::None,
    }
}
