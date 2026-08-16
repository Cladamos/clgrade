use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    buffer::Buffer,
    crossterm,
    layout::{Constraint, Direction, Layout, Rect, Size},
    widgets::Widget,
};
use ratatui_image::{FilterType::Nearest, Image, Resize, picker::Picker, protocol::Protocol};

pub struct App {
    image: Protocol,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        let picker = Picker::from_query_stdio().unwrap();
        let dyn_img = image::ImageReader::open("src/test.png")
            .unwrap()
            .decode()
            .unwrap();

        let target_size = Size::new(30, 30);

        let image = picker
            .new_protocol(dyn_img, target_size, Resize::Fit(Some(Nearest)))
            .unwrap();

        App { exit: false, image }
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        frame.render_widget(self, frame.area());
    }

    /// updates the application's state based on user input
    fn handle_events(&mut self) -> io::Result<()> {
        match event::read()? {
            // it's important to check that the event is a key press event as
            // crossterm also emits key release and repeat events on Windows.
            Event::Key(key_event) if key_event.kind == KeyEventKind::Press => {
                self.handle_key_event(key_event)
            }
            _ => {}
        };
        Ok(())
    }

    fn handle_key_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char('q') => self.exit(),
            _ => {}
        }
    }
    fn exit(&mut self) {
        self.exit = true;
    }
}

impl Widget for &App {
    fn render(self, area: Rect, buf: &mut Buffer) {
        Image::new(&self.image)
            .allow_clipping(true)
            .render(area, buf);
    }
}
