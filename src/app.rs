use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm,
    layout::{Rect, Size},
    widgets::{FrameExt, Widget},
};
use ratatui_explorer::{FileExplorer, FileExplorerBuilder, Theme};
use ratatui_image::{FilterType::Nearest, Image, Resize, picker::Picker, protocol::Protocol};

pub struct App {
    image: Option<Protocol>,
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

        App {
            exit: false,
            is_file_explorer_visible: false,
            is_image_selected: false,
            file_explorer: file_explorer,
            image: None,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;

            let picker = Picker::from_query_stdio().unwrap();

            if self.is_image_selected
                && let Some(img_path) = self.file_explorer.current().path.to_str()
            {
                if img_path.ends_with(".png")
                    || img_path.ends_with(".jpg")
                    || img_path.ends_with(".jpeg")
                    || img_path.ends_with(".webp")
                {
                    let dyn_img = image::ImageReader::open(img_path)
                        .unwrap()
                        .decode()
                        .unwrap();

                    let target_size = Size::new(40, 40);

                    self.image = Some(
                        picker
                            .new_protocol(dyn_img, target_size, Resize::Fit(Some(Nearest)))
                            .unwrap(),
                    );

                    self.is_image_selected = false;
                }
            }
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        if self.is_file_explorer_visible {
            frame.render_widget_ref(self.file_explorer.widget(), frame.area());
        } else {
            frame.render_widget(self, frame.area());
        }
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        let event = event::read()?;
        match event {
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };

        if self.is_file_explorer_visible {
            self.file_explorer.handle(&event)?;
        }
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            KeyCode::Char('f') => self.is_file_explorer_visible = !self.is_file_explorer_visible,
            KeyCode::Enter => {
                if self.is_file_explorer_visible == true {
                    self.is_image_selected = true;
                    self.is_file_explorer_visible = false;
                }
            }
            _ => {}
        }
    }
    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        if let Some(image) = self.image.as_ref() {
            Image::new(image).allow_clipping(true).render(area, buf);
        }
    }
}
