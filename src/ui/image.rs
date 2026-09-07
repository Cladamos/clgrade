use crate::app::Status;
use crate::image::ImageHandler;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Margin, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::text::{Span, Text};
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Borders, Widget};
use ratatui_image::Image;

pub struct ImageSection<'a> {
    image_handler: &'a ImageHandler,
    pub aspect_ratio: (u8, u8),
    pub resolution: u32,
    pub export_status: Option<Status>,
    image_area: Rect,
}

impl<'a> ImageSection<'a> {
    pub fn new(image_handler: &'a ImageHandler, image_area: Rect) -> Self {
        ImageSection {
            image_handler,
            aspect_ratio: (1, 1),
            resolution: 240,
            export_status: None,
            image_area,
        }
    }
}

impl<'a> Widget for ImageSection<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let image_section_area = self.image_area;
        let image_area = Rect {
            x: image_section_area.x,
            y: image_section_area.y,
            width: image_section_area.width,
            height: image_section_area.height.saturating_sub(1),
        };
        let text_area = Rect {
            x: area.x,
            y: image_section_area.y + image_section_area.height.saturating_sub(1),
            width: area.width,
            height: 1,
        };

        if let Some(protocol) = self.image_handler.protocol.as_ref() {
            Image::new(protocol).render(image_area.inner(Margin::new(1, 1)), buf);
        }

        if let Some(status) = self.export_status {
            let span = match status {
                Status::Success(msg) => Span::styled(
                    msg,
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD),
                ),
                Status::Error(msg) => Span::styled(
                    msg,
                    Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
                ),
            };
            Text::from(span)
                .alignment(Alignment::Center)
                .render(text_area, buf);
        } else {
            let bottom_text =
                if self.image_handler.protocol.is_none() && !self.image_handler.loading {
                    "File Explorer: 'f'"
                }
                // TODO: add loading animation, instead of plain text.
                else if self.image_handler.loading {
                    "Loading..."
                } else {
                    ""
                };
            Text::from(bottom_text)
                .fg(Color::DarkGray)
                .alignment(Alignment::Center)
                .render(text_area, buf);
        }

        let aspect_ratio = format!("{}:{}", self.aspect_ratio.0, self.aspect_ratio.1);
        let resolution = format!("{}p", self.resolution);
        let title = format!(" {} {} ", aspect_ratio, resolution);
        Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(Rounded)
            .render(image_area, buf);
    }
}
