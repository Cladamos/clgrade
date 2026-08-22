use crate::image::ImageHandler;
use ratatui::buffer::Buffer;
use ratatui::layout::{Constraint, Margin, Rect};
use ratatui::style::{Color, Stylize};
use ratatui::text::Text;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Borders, Widget};
use ratatui_image::Image;

pub struct ImageSection<'a> {
    pub image_handler: &'a ImageHandler,
}

impl<'a> ImageSection<'a> {
    pub fn new(image_handler: &'a ImageHandler) -> Self {
        ImageSection { image_handler }
    }
}

impl<'a> Widget for ImageSection<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(protocol) = self.image_handler.protocol.as_ref() {
            Image::new(protocol).render(area.inner(Margin::new(1, 1)), buf);
        }

        if self.image_handler.protocol.is_none() && !self.image_handler.loading {
            Text::from("Select an image with 'f' to open file explorer.")
                .fg(Color::DarkGray)
                .render(
                    area.centered(Constraint::Length(47), Constraint::Length(1)),
                    buf,
                );
        }

        // TODO: add loading animation, instead of plain text.
        if self.image_handler.loading {
            Text::from("Loading...").fg(Color::DarkGray).render(
                area.centered(Constraint::Length(10), Constraint::Length(1)),
                buf,
            );
        }

        Block::default()
            .title("Image")
            .borders(Borders::ALL)
            .border_type(Rounded)
            .render(area, buf);
    }
}
