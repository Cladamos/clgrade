use std::io;
use std::sync::mpsc;
use std::thread;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm,
    layout::{Constraint, Margin, Rect, Size},
    style::{Color, Stylize},
    text::Text,
    widgets::{Block, BorderType::Rounded, Borders, FrameExt, Widget},
};
use ratatui_explorer::{FileExplorer, FileExplorerBuilder, Theme};
use ratatui_image::{FilterType::Nearest, Image, Resize, picker::Picker, protocol::Protocol};

pub struct App {
    image: Option<Protocol>,
    image_rx: mpsc::Receiver<Protocol>,
    image_tx: mpsc::Sender<Protocol>,
    loading: bool,
    is_file_explorer_visible: bool,
    is_image_selected: bool,
    file_explorer: FileExplorer,
    exit: bool,
}

const SUPPORTED_FORMATS: &[&str] = &["png", "jpg", "jpeg", "webp"];

impl App {
    pub fn new() -> Self {
        let (image_tx, image_rx) = mpsc::channel();
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
            image: None,
            image_rx: image_rx,
            image_tx: image_tx,
            loading: false,
            exit: false,
            is_file_explorer_visible: false,
            is_image_selected: false,
            file_explorer: file_explorer,
        }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            if self.is_image_selected {
                self.draw_image();
            }
            if let Ok(protocol) = self.image_rx.try_recv() {
                self.image = Some(protocol);
                self.loading = false;
            }
        }
        Ok(())
    }

    fn draw_image(&mut self) {
        let tx = self.image_tx.clone();
        let picker = Picker::from_query_stdio().unwrap();
        let img_path = self.file_explorer.current().path.clone();
        self.loading = true;
        thread::spawn(move || {
            let dyn_img = image::ImageReader::open(img_path)
                .unwrap()
                .decode()
                .unwrap();

            let target_size = Size::new(60, 30);

            let image_protocol = picker
                .new_protocol(dyn_img, target_size, Resize::Fit(Some(Nearest)))
                .unwrap();

            tx.send(image_protocol).unwrap();
        });
        self.is_image_selected = false;
    }

    fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(16))? {
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
            _ => {}
        }
    }
    fn draw(&self, frame: &mut Frame) {
        if self.is_file_explorer_visible {
            frame.render_widget_ref(self.file_explorer.widget(), frame.area());
        } else {
            frame.render_widget(self, frame.area());
        }
    }
    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        let image_area = Rect {
            x: area.x,
            y: area.y,
            width: 62,
            height: 32,
        };

        if let Some(image) = self.image.as_ref() {
            Image::new(image).allow_clipping(true).render(
                image_area.inner(Margin::new(1, 1)).centered(
                    Constraint::Length(image.size().width),
                    Constraint::Length(image.size().height),
                ),
                buf,
            );
        }
        if self.image.is_none() && !self.loading {
            Text::from("Select an image with 'f' to open file explorer.")
                .fg(Color::DarkGray)
                .render(
                    image_area.centered(Constraint::Length(47), Constraint::Length(1)),
                    buf,
                );
        }
        // TODO: add loading animation, instead of plain text.
        if self.loading {
            Text::from("Loading...").fg(Color::DarkGray).render(
                image_area.centered(Constraint::Length(10), Constraint::Length(1)),
                buf,
            );
        }
        Block::default()
            .title("Image")
            .borders(Borders::ALL)
            .border_type(Rounded)
            .render(image_area, buf);
    }
}
