use crate::image::ImageHandler;
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Constraint, Direction, Layout, Margin, Rect};
use ratatui::style::{Color, Stylize};
use ratatui::text::Text;
use ratatui::widgets::BorderType::Rounded;
use ratatui::widgets::{Block, Borders, Widget};
use ratatui_image::Image;

pub struct ImageSection<'a> {
    pub image_handler: &'a ImageHandler,
    pub aspect_ratio: (u8, u8),
    pub resolution: u32,
}

impl<'a> ImageSection<'a> {
    pub fn new(image_handler: &'a ImageHandler) -> Self {
        ImageSection {
            image_handler,
            aspect_ratio: (1, 1),
            resolution: 240,
        }
    }
}

impl<'a> Widget for ImageSection<'a> {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let image_layout = Layout::default()
            .direction(Direction::Vertical)
            .constraints([
                Constraint::Length(area.height - 1), // image
                Constraint::Length(1),               // explorer help message
            ])
            .split(area);

        if let Some(protocol) = self.image_handler.protocol.as_ref() {
            Image::new(protocol).render(image_layout[0].inner(Margin::new(1, 1)), buf);
        }

        if self.image_handler.protocol.is_none() && !self.image_handler.loading {
            Text::from("File Explorer: 'f'")
                .fg(Color::DarkGray)
                .alignment(Alignment::Center)
                .render(image_layout[1], buf);
        }

        // TODO: add loading animation, instead of plain text.
        if self.image_handler.loading {
            Text::from("Loading...")
                .fg(Color::DarkGray)
                .alignment(Alignment::Center)
                .render(image_layout[1], buf);
        }

        let aspect_ratio = format!("{}:{}", self.aspect_ratio.0, self.aspect_ratio.1);
        let resolution = format!("{}p", self.resolution);
        let title = format!(" {} {} ", aspect_ratio, resolution);
        Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_type(Rounded)
            .render(image_layout[0], buf);
    }
}
