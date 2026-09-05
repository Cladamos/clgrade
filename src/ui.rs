pub mod help;
pub mod image;
pub mod pipeline;
pub mod preset;
pub mod scope;
pub mod slider;
pub mod wheel;

use ratatui::{
    layout::{Alignment, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span, Text},
    widgets::{Block, BorderType::Rounded, Borders},
};
use ratatui_explorer::Theme;

use crate::app::ActivePage;

#[derive(Debug, Copy, Clone)]
pub struct CenterOpts {
    pub width: u16,
    pub height: u16,
    pub margin: u16,
}
pub fn centered_rect(opts: CenterOpts, r: Rect) -> Rect {
    let padding_vertical = r.height.saturating_sub(opts.height) / 2;
    let padding_horizontal = r.width.saturating_sub(opts.width) / 2;

    Rect {
        x: r.x + padding_horizontal,
        y: r.y + padding_vertical,
        width: opts.width.min(r.width),
        height: opts.height.min(r.height),
    }
    .inner(Margin {
        horizontal: opts.margin,
        vertical: 0,
    })
}

pub fn focused_style(selected: bool) -> Style {
    if selected {
        Style::default()
            .fg(Color::Blue)
            .add_modifier(Modifier::BOLD)
    } else {
        Style::default().fg(Color::Gray)
    }
}

pub fn focused_color(selected: bool) -> Color {
    if selected { Color::Blue } else { Color::Gray }
}

pub fn warning_msg<'a>(msg: &'a str) -> Text<'a> {
    Text::from(msg)
        .style(Color::Red)
        .alignment(Alignment::Center)
}

pub fn file_explorer_theme() -> Theme {
    let block = Block::default().borders(Borders::ALL).border_type(Rounded);
    Theme::default()
        .with_block(block)
        .add_default_title()
        .with_title_bottom(|_| "<f>: close | <enter>: select | <s>: save".into())
}

pub fn preset_explorer_theme() -> Theme {
    let block = Block::default().borders(Borders::ALL).border_type(Rounded);
    Theme::default()
        .with_block(block)
        .add_default_title()
        .with_title_bottom(|_| "<enter>: select | <s>: save | <d/del>: delete".into())
}

pub fn page_indicator<'a>(page: ActivePage) -> Line<'a> {
    let default_style = Style::default().fg(Color::DarkGray);
    let selected_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let pages: [(&str, ActivePage); 5] = [
        ("1: sliders", ActivePage::Sliders),
        ("2: wheels", ActivePage::Wheels),
        ("3: scopes", ActivePage::Scopes),
        ("4: pipeline", ActivePage::Pipeline),
        ("5: presets", ActivePage::Preset),
    ];

    Line::from(
        pages
            .iter()
            .enumerate()
            .map(|(i, (name, active_page))| {
                let s = if *active_page == page {
                    selected_style
                } else {
                    default_style
                };
                vec![
                    Span::styled(format!("{}", name), s),
                    if i == pages.len() - 1 {
                        Span::from("")
                    } else {
                        Span::from(" | ")
                    },
                ]
            })
            .flatten()
            .collect::<Vec<Span>>(),
    )
    .alignment(Alignment::Center)
}
