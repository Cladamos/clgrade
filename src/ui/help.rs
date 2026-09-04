use ratatui::{
    buffer::Buffer,
    layout::{Constraint, Direction, Layout, Margin, Rect},
    style::{Color, Modifier, Style},
    text::{Line, Span},
    widgets::{Block, BorderType::Rounded, Borders, Paragraph, Widget},
};

use super::{CenterOpts, centered_rect, page_indicator};
use crate::app::ActivePage;

pub struct HelpSection {}
impl HelpSection {
    const WIDTH: u16 = 74;
    const HEIGHT: u16 = 20;
}

impl HelpSection {
    fn section_header(title: &'static str) -> Line<'static> {
        Line::from(vec![
            Span::styled(" ", Style::default()),
            Span::styled(
                title,
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
        ])
    }

    fn key_entry(key: &'static str, desc: &'static str) -> Line<'static> {
        Line::from(vec![
            Span::styled(
                format!("  {:<8}", key),
                Style::default()
                    .fg(Color::Blue)
                    .add_modifier(Modifier::BOLD),
            ),
            Span::from(desc),
        ])
    }
}

impl Widget for HelpSection {
    fn render(self, area: Rect, buf: &mut Buffer) {
        // Bottom page indicator
        let footer_area = Rect {
            x: area.x,
            y: area.bottom().saturating_sub(1),
            width: area.width,
            height: 1,
        };
        page_indicator(ActivePage::Help).render(footer_area, buf);

        let main_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: area.height.saturating_sub(1),
        };

        let container_width = HelpSection::WIDTH;
        let container_height = HelpSection::HEIGHT;

        let help_area = centered_rect(
            CenterOpts {
                width: container_width,
                height: container_height,
                margin: 0,
            },
            main_area,
        );

        let block = Block::default()
            .borders(Borders::ALL)
            .border_type(Rounded)
            .title(Line::from(vec![
                Span::styled(
                    " clgrade ",
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                Span::from("• Keyboard Controls "),
            ]));

        let inner_area = block.inner(help_area.inner(Margin::new(2, 1)));
        block.render(help_area, buf);

        let left_col = vec![
            Self::section_header("Navigation"),
            Self::key_entry("1 - 4", "Switch View (1-4)"),
            Self::key_entry("Tab", "Change Selected Part"),
            Self::key_entry("?", "Show Help Screen"),
            Line::default(),
            Self::section_header("Adjustments"),
            Self::key_entry("↑ / k", "Increase / Move Up"),
            Self::key_entry("→ / l", "Increase / Move Right"),
            Self::key_entry("↓ / j", "Decrease / Move Down"),
            Self::key_entry("← / h", "Decrease / Move Left"),
            Self::key_entry("r", "Reset Selected Tool"),
            Self::key_entry("R", "Reset All"),
            Line::default(),
            Self::section_header("General"),
            Self::key_entry("o", "Switch Layout"),
            Self::key_entry("q / ^c", "Quit Application"),
        ];

        let right_col = vec![
            Self::section_header("Preview"),
            Self::key_entry("Space", "Hold: View Original"),
            Self::key_entry("p", "Toggle Proxy Mode"),
            Self::key_entry("a", "Change Aspect Ratio"),
            Self::key_entry("A", "Change Resolution"),
            Line::default(),
            Self::section_header("Files & Export"),
            Self::key_entry("f", "Toggle File Explorer"),
            Self::key_entry("Enter", "Select / Open Image"),
            Self::key_entry("Ctrl+s", "Export Image (Picker)"),
            Self::key_entry("s", "Save to Directory"),
        ];

        let cols = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(50), Constraint::Percentage(50)])
            .split(inner_area);

        Paragraph::new(left_col).render(cols[0], buf);
        Paragraph::new(right_col).render(cols[1], buf);
    }
}
