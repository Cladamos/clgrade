pub mod color_mixer;
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

pub enum ExplorerType {
    File,
    Preset,
}

pub fn explorer_theme(explorer_type: ExplorerType) -> Theme {
    let block = Block::default().borders(Borders::ALL).border_type(Rounded);
    let bottom_title = match explorer_type {
        ExplorerType::File => "<f>: close | <enter>: select | <s>: save",
        ExplorerType::Preset => "<enter>: select | <s>: save | <d/del>: delete",
    };
    Theme::default()
        .with_block(block)
        .add_default_title()
        .with_title_bottom(|_| bottom_title.into())
}

pub fn page_indicator<'a>(page: ActivePage, width: u16) -> Line<'a> {
    let default_style = Style::default().fg(Color::DarkGray);
    let selected_style = Style::default()
        .fg(Color::White)
        .add_modifier(Modifier::BOLD);
    let pages: [(&str, ActivePage); 6] = [
        ("1: sliders", ActivePage::Sliders),
        ("2: wheels", ActivePage::Wheels),
        ("3: scopes", ActivePage::Scopes),
        ("4: pipeline", ActivePage::Pipeline),
        ("5: presets", ActivePage::Preset),
        ("6: color mixer", ActivePage::ColorMixer),
    ];
    let mut text_len: usize = 0;
    let mut page_text = pages
        .iter()
        .enumerate()
        .flat_map(|(i, (name, active_page))| {
            let s = if *active_page == page {
                selected_style
            } else {
                default_style
            };

            text_len += name.len();
            text_len += if i == pages.len() - 1 { 1 } else { 3 };
            vec![
                Span::styled(*name, s),
                if i == pages.len() - 1 {
                    Span::from("")
                } else {
                    Span::from(" | ")
                },
            ]
        })
        .collect::<Vec<Span>>();
    if text_len > width as usize {
        page_text = vec![Span::styled("[1-6]: pages | ?: help", default_style)];
    }
    Line::from(page_text).alignment(Alignment::Center)
}
