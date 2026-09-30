use crate::app::Status;
use crate::effect::{CropArea, CropCorner};
use crate::image::ImageHandler;
use crate::ui::{CenterOpts, centered_rect};
use ratatui::buffer::Buffer;
use ratatui::layout::{Alignment, Margin, Rect};
use ratatui::style::{Color, Modifier, Style, Stylize};
use ratatui::symbols::border;
use ratatui::text::{Span, Text};
use ratatui::widgets::{Block, Borders, Widget};
use ratatui_image::Image;

pub struct ImageSection<'a> {
    image_handler: &'a ImageHandler,
    pub aspect_ratio: (u8, u8),
    pub resolution: u32,
    pub export_status: Option<Status>,
    pub crop_area: CropArea,
    pub crop_corner: CropCorner,
    image_area: Rect,
    image_center_opts: CenterOpts,
    is_crop_mode: bool,
}

impl<'a> ImageSection<'a> {
    pub fn new(
        image_handler: &'a ImageHandler,
        image_area: Rect,
        image_center_opts: CenterOpts,
        is_crop_mode: bool,
        crop_area: CropArea,
        crop_corner: CropCorner,
    ) -> Self {
        ImageSection {
            image_handler,
            aspect_ratio: (1, 1),
            resolution: 240,
            export_status: None,
            crop_area,
            crop_corner,
            image_area,
            image_center_opts,
            is_crop_mode,
        }
    }
}

impl<'a> Widget for ImageSection<'a> {
    fn render(self, _: Rect, buf: &mut Buffer) {
        let image_area_centered = centered_rect(self.image_center_opts, self.image_area);
        let image = Rect {
            x: image_area_centered.x,
            y: image_area_centered.y,
            width: image_area_centered.width,
            height: image_area_centered.height.saturating_sub(1),
        };
        let text_area = Rect {
            x: self.image_area.x,
            y: image_area_centered.y + image_area_centered.height.saturating_sub(1),
            width: self.image_area.width,
            height: 1,
        };

        if let Some(protocol) = self.image_handler.protocol.as_ref() {
            Image::new(protocol).render(image.inner(Margin::new(1, 1)), buf);
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
        } else if let Some(ref error) = self.image_handler.load_error {
            Text::from(Span::styled(
                error.as_str(),
                Style::default().fg(Color::Red).add_modifier(Modifier::BOLD),
            ))
            .alignment(Alignment::Center)
            .render(text_area, buf);
        } else {
            let bottom_text =
                if self.image_handler.protocol.is_none() && !self.image_handler.loading {
                    String::from("f: file explorer | ?: help")
                }
                // TODO: add loading animation, instead of plain text.
                else if self.image_handler.loading {
                    String::from("Loading...")
                } else if self.is_crop_mode {
                    format!(
                        "Crop [{}]: {}x{} (x:{}, y:{}) | tab: corner | enter: confirm",
                        self.crop_corner.label(),
                        self.crop_area.width,
                        self.crop_area.height,
                        self.crop_area.x,
                        self.crop_area.y
                    )
                } else {
                    String::new()
                };
            Text::from(bottom_text)
                .fg(Color::DarkGray)
                .alignment(Alignment::Center)
                .render(text_area, buf);
        }

        let aspect_ratio = format!("{}:{}", self.aspect_ratio.0, self.aspect_ratio.1);
        let resolution = format!("{}p", self.resolution);
        let title = format!(" {} {} ", aspect_ratio, resolution);

        let border_set = if self.is_crop_mode {
            let mut set = border::ROUNDED;
            match self.crop_corner {
                CropCorner::TopLeft => set.top_left = "●",
                CropCorner::BottomRight => set.bottom_right = "●",
            }
            set
        } else {
            border::ROUNDED
        };

        Block::default()
            .title(title)
            .title_alignment(Alignment::Center)
            .borders(Borders::ALL)
            .border_set(border_set)
            .render(image, buf);
    }
}
