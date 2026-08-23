pub mod image;
pub mod slider;

use ratatui::{
    layout::{Alignment, Margin, Rect},
    style::Color,
    text::Text,
};

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
