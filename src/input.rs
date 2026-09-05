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
    SwitchToScopes,
    SwitchToPipeline,
    SwitchToPreset,
    ChangeAspectRatio,
    ChangeResolution,
    ResetAll,
    Select,
    Delete,
    Save,
    ToggleOriginal,
    ToggleProxy,
    ToggleLayout,
    ToggleHelp,
    Escape,
    None,
}

pub fn map_key_to_action(key: KeyEvent) -> Action {
    match (key.modifiers, key.code) {
        // Navigation
        (_, KeyCode::Char('1')) => Action::SwitchToSliders,
        (_, KeyCode::Char('2')) => Action::SwitchToWheels,
        (_, KeyCode::Char('3')) => Action::SwitchToScopes,
        (_, KeyCode::Char('4')) => Action::SwitchToPipeline,
        (_, KeyCode::Char('5')) => Action::SwitchToPreset,
        (_, KeyCode::Tab) => Action::NextTool,
        (_, KeyCode::Char('?')) => Action::ToggleHelp,

        // Adjustments
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
        (_, KeyCode::Char('r')) => Action::ResetTool,
        (_, KeyCode::Char('R')) => Action::ResetAll,

        // General
        (_, KeyCode::Char('o')) => Action::ToggleLayout,
        (_, KeyCode::Char('q')) | (KeyModifiers::CONTROL, KeyCode::Char('c')) => Action::Quit,
        (_, KeyCode::Esc) => Action::Escape,

        // Preview
        (_, KeyCode::Char(' ')) => Action::ToggleOriginal,
        (_, KeyCode::Char('p')) => Action::ToggleProxy,
        (_, KeyCode::Char('a')) => Action::ChangeAspectRatio,
        (_, KeyCode::Char('A')) => Action::ChangeResolution,

        // File
        (_, KeyCode::Char('f')) => Action::ToggleFileExplorer,
        (_, KeyCode::Enter) => Action::Select,
        (KeyModifiers::CONTROL, KeyCode::Char('s')) => Action::ExportImage,
        (_, KeyCode::Char('s')) => Action::Save,
        (_, KeyCode::Delete | KeyCode::Char('d')) => Action::Delete,

        _ => Action::None,
    }
}
