use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Rect};
use ratatui::style::{Color, Modifier, Style};
use ratatui::text::{Line, Span};
use ratatui::widgets::{Block, BorderType::Rounded, Borders, Paragraph, Widget};

use crate::app::AppLayout;

pub struct PresetSection<'a> {
    pub input: &'a str,
    pub is_input_mode: bool,
    pub status: Option<&'a str>,
}

impl<'a> PresetSection<'a> {
    pub const INPUT_HEIGHT: u16 = 3;
    pub const EXPLORER_HEIGHT: u16 = 12;

    pub fn new(input: &'a str, is_input_mode: bool, status: Option<&'a str>) -> Self {
        Self {
            input,
            is_input_mode,
            status,
        }
    }

    pub fn layout(area: Rect, app_layout: AppLayout) -> (Rect, Rect) {
        let (area, constraints) = match app_layout {
            AppLayout::Vertical => (
                area,
                vec![
                    Constraint::Length(Self::EXPLORER_HEIGHT),
                    Constraint::Length(Self::INPUT_HEIGHT),
                ],
            ),
            AppLayout::Horizontal => (
                Rect {
                    x: area.x,
                    y: area.y,
                    width: area.width,
                    height: area.height.saturating_sub(2),
                },
                vec![
                    Constraint::Length(area.height.saturating_sub(Self::INPUT_HEIGHT)),
                    Constraint::Length(Self::INPUT_HEIGHT),
                ],
            ),
        };
        let chunks = Layout::default()
            .direction(Direction::Vertical)
            .constraints(constraints)
            .split(area);
        (chunks[0], chunks[1])
    }
}

impl Widget for PresetSection<'_> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let border_style = if self.is_input_mode {
            Style::default()
                .fg(Color::Blue)
                .add_modifier(Modifier::BOLD)
        } else {
            Style::default().fg(Color::DarkGray)
        };

        let bottom_title = if self.is_input_mode {
            "<enter>: confirm | <esc>: cancel"
        } else {
            ""
        };

        let block = Block::default()
            .title_bottom(bottom_title)
            .borders(Borders::ALL)
            .border_type(Rounded)
            .border_style(border_style);

        let content: Line = if self.is_input_mode {
            Line::from(vec![
                Span::styled(
                    self.input,
                    Style::default()
                        .fg(Color::White)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::styled(
                    "▎",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::SLOW_BLINK),
                ),
            ])
        } else if let Some(status) = self.status {
            Line::from(Span::styled(
                status,
                if status.starts_with("Error:") {
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD)
                } else {
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                },
            ))
            .alignment(Alignment::Center)
        } else {
            Line::from(Span::styled(
                "Select a preset to load",
                Style::default().fg(Color::DarkGray),
            ))
            .alignment(Alignment::Center)
        };

        Paragraph::new(content).block(block).render(area, buf);
    }
}
