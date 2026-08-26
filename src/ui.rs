pub mod image;
pub mod slider;
pub mod wheel;

use ratatui::{
    layout::{Alignment, Margin, Rect},
    style::{Color, Style},
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

pub fn page_indicator<'a>(page: ActivePage) -> Line<'a> {
    let mut sliders = Span::styled("<1>: sliders ", Style::default().fg(Color::DarkGray));
    let mut wheels = Span::styled("<2>: wheels", Style::default().fg(Color::DarkGray));

    match page {
        ActivePage::Sliders => sliders = sliders.style(Style::default().fg(Color::White).bold()),
        ActivePage::Wheels => wheels = wheels.style(Style::default().fg(Color::White).bold()),
    }
    Line::from(vec![sliders, " | ".into(), wheels]).alignment(Alignment::Center)
}
