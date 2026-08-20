use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame, crossterm,
    layout::{Constraint, Margin, Rect},
    style::{Color, Stylize},
    text::Text,
    widgets::{Block, BorderType::Rounded, Borders, FrameExt, Widget},
};
use ratatui_explorer::{FileExplorer, FileExplorerBuilder, Theme};
use ratatui_image::Image;
use tui_slider::{Slider, SliderOrientation, SliderState};

use crate::image::{ColorGrade, ImageHandler};

pub struct App {
    image_handler: ImageHandler,
    hue: SliderState,
    is_re_render: bool,
    is_file_explorer_visible: bool,
    is_image_selected: bool,
    file_explorer: FileExplorer,
    exit: bool,
}

const SUPPORTED_FORMATS: &[&str] = &["png", "jpg", "jpeg", "webp"];

impl App {
    pub fn new() -> Self {
        let theme = Theme::default().add_default_title();
        let mut file_explorer = FileExplorerBuilder::build_with_theme(theme).unwrap();
        file_explorer
            .set_filter_map(|file| {
                let keep = match file.path.extension() {
                    Some(extension) => {
                        let extension = extension.to_str().unwrap_or_default();
                        SUPPORTED_FORMATS.contains(&extension)
                    }
                    None => file.is_dir,
                };

                if keep { Some(file) } else { None }
            })
            .unwrap();
        let hue_slider = SliderState::new(0.0, 0.0, 360.0);

        App {
            image_handler: ImageHandler::new(),
            hue: hue_slider,
            is_re_render: false,
            is_file_explorer_visible: false,
            is_image_selected: false,
            file_explorer: file_explorer,
            exit: false,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            if self.is_image_selected {
                let path = self.file_explorer.current().path.clone();
                self.image_handler.load_from_path(path);
                self.is_image_selected = false;
            }
            if self.is_re_render && self.image_handler.has_source() {
                self.image_handler.apply_effects(ColorGrade {
                    hue_degrees: self.hue.value() as f32,
                });
                self.is_re_render = false;
            }
            self.image_handler.poll();
        }
        Ok(())
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(16))? {
            let event = event::read()?;
            match event {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                    self.handle_key_event(key_event)
                }
                _ => {}
            }

            if self.is_file_explorer_visible {
                self.file_explorer.handle(&event)?;
            }
        }

        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('f') => self.is_file_explorer_visible = !self.is_file_explorer_visible,
            KeyCode::Enter => {
                if self.is_file_explorer_visible && self.file_explorer.current().is_dir == false {
                    self.is_image_selected = true;
                    self.is_file_explorer_visible = false;
                }
            }
            KeyCode::Up => {
                self.hue.increase(1.0);
                self.is_re_render = true;
            }
            KeyCode::Down => {
                self.hue.decrease(1.0);
                self.is_re_render = true;
            }
            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame) {
        if self.is_file_explorer_visible {
            frame.render_widget_ref(self.file_explorer.widget(), frame.area());
            return;
        }

        let area = frame.area();

        let image_area = Rect {
            x: area.x,
            y: area.y,
            width: 62,
            height: 30,
        };

        let slider_area = Rect {
            x: area.x + image_area.width + 1,
            y: area.y,
            width: 60,
            height: 28,
        };

        if let Some(protocol) = self.image_handler.protocol.as_ref() {
            Image::new(protocol).render(image_area.inner(Margin::new(1, 1)), frame.buffer_mut());
        }

        if self.image_handler.protocol.is_none() && !self.image_handler.loading {
            Text::from("Select an image with 'f' to open file explorer.")
                .fg(Color::DarkGray)
                .render(
                    image_area.centered(Constraint::Length(47), Constraint::Length(1)),
                    frame.buffer_mut(),
                );
        }

        // TODO: add loading animation, instead of plain text.
        if self.image_handler.loading {
            Text::from("Loading...").fg(Color::DarkGray).render(
                image_area.centered(Constraint::Length(10), Constraint::Length(1)),
                frame.buffer_mut(),
            );
        }

        Block::default()
            .title("Image")
            .borders(Borders::ALL)
            .border_type(Rounded)
            .render(image_area, frame.buffer_mut());

        let slider = Slider::from_state(&self.hue)
            .orientation(SliderOrientation::Vertical)
            .label("Hue")
            .show_value(true);
        slider.render(slider_area, frame.buffer_mut());
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
