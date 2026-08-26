use crossterm::event::{KeyCode, KeyEvent, KeyModifiers};
use ratatui::crossterm;

pub enum Action {
    Quit,
    ExportImage,
    ToggleFileExplorer,
    NextTool,
    AdjustValue { delta_x: f32, delta_y: f32 },
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
    ToggleLayout,
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
        (_, KeyCode::Char('o')) => Action::ToggleLayout,

        // Navigation
        (_, KeyCode::Tab) => Action::NextTool,
        (_, KeyCode::Char('1')) => Action::SwitchToSliders,
        (_, KeyCode::Char('2')) => Action::SwitchToWheels,

        // Manipulation
        (_, KeyCode::Up) | (_, KeyCode::Char('k')) => Action::AdjustValue {
            delta_x: 0.0,
            delta_y: 1.0,
        },
        (_, KeyCode::Down) | (_, KeyCode::Char('j')) => Action::AdjustValue {
            delta_x: 0.0,
            delta_y: -1.0,
        },
        (_, KeyCode::Right) | (_, KeyCode::Char('l')) => Action::AdjustValue {
            delta_x: 1.0,
            delta_y: 0.0,
        },
        (_, KeyCode::Left) | (_, KeyCode::Char('h')) => Action::AdjustValue {
            delta_x: -1.0,
            delta_y: 0.0,
        },
        (_, KeyCode::Char('a')) => Action::ChangeAspectRatio,
        (_, KeyCode::Char('A')) => Action::ChangeResolution,
        (_, KeyCode::Char('r')) => Action::ResetTool,
        (_, KeyCode::Char('R')) => Action::ResetAll,

        _ => Action::None,
    }
}
