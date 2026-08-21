use std::io;

use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame, crossterm,
    layout::{Constraint, Direction, Layout, Margin, Rect, Size},
    style::{Color, Modifier, Style, Stylize},
    text::Text,
    widgets::{Block, BorderType::Rounded, Borders, FrameExt, Widget},
};
use ratatui_explorer::{FileExplorer, FileExplorerBuilder, Theme};
use ratatui_image::Image;
use tui_slider::{Slider, SliderOrientation, SliderState, style::SliderStyle};

use crate::image::{ColorGrade, ImageHandler};

pub struct App {
    image_handler: ImageHandler,
    sliders: Vec<SliderData>,
    selected_slider_index: usize,
    is_re_render: bool,
    is_file_explorer_visible: bool,
    is_image_selected: bool,
    file_explorer: FileExplorer,
    exit: bool,
}

struct SliderData {
    label: String,
    state: SliderState,
    step: f64,
    style: SliderStyle,
    is_selected: bool,
}
const SUPPORTED_FORMATS: &[&str] = &["png", "jpg", "jpeg", "webp"];

impl App {
    pub fn new() -> Self {
        let sliders: Vec<SliderData> = vec![
            SliderData {
                label: "Temp".to_string(),
                state: SliderState::new(0.0, -100.0, 100.0),
                step: 5.0,
                style: SliderStyle::vertical(),
                is_selected: true,
            },
            SliderData {
                label: "Exp".to_string(),
                state: SliderState::new(0.0, -3.0, 3.0),
                step: 0.1,
                style: SliderStyle::vertical(),
                is_selected: false,
            },
            SliderData {
                label: "Cont".to_string(),
                state: SliderState::new(0.0, -100.0, 100.0),
                step: 5.0,
                style: SliderStyle::vertical(),
                is_selected: false,
            },
            SliderData {
                label: "Sat".to_string(),
                state: SliderState::new(1.0, 0.0, 2.0),
                step: 0.1,
                style: SliderStyle::vertical(),
                is_selected: false,
            },
            SliderData {
                label: "Hue".to_string(),
                state: SliderState::new(0.0, -180.0, 180.0),
                step: 5.0,
                style: SliderStyle::vertical(),
                is_selected: false,
            },
        ];
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
            image_handler: ImageHandler::new(),
            sliders,
            selected_slider_index: 0,
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
                    hue_degrees: self.sliders[4].state.value() as f32,
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
            KeyCode::Tab => {
                self.sliders[self.selected_slider_index].is_selected = false;
                self.selected_slider_index = (self.selected_slider_index + 1) % self.sliders.len();
                self.sliders[self.selected_slider_index].is_selected = true;
            }
            KeyCode::Up => {
                let s = &mut self.sliders[self.selected_slider_index];
                s.state.increase(s.step);
                self.is_re_render = true;
            }
            KeyCode::Down => {
                let s = &mut self.sliders[self.selected_slider_index];
                s.state.decrease(s.step);
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

        let image_size = self
            .image_handler
            .protocol
            .as_ref()
            .map(|p| p.size())
            .unwrap_or(Size::new(60, 28));

        let image_area = Rect {
            x: area.x,
            y: area.y,
            width: image_size.width + 2,
            height: image_size.height + 2,
        };

        let slider_area = Rect {
            x: area.x,
            y: area.y + image_area.height + 2,
            width: area.width,
            height: 10,
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

        // TODO: make it responsive for terminal resize.
        // TODO: add horizontal split option
        let slider_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints(vec![
                Constraint::Length(15),
                Constraint::Length(15),
                Constraint::Length(15),
                Constraint::Length(15),
                Constraint::Length(15),
            ])
            .split(slider_area);

        self.sliders.iter().enumerate().for_each(|(index, slider)| {
            Block::default()
                .title(ratatui::text::Line::from(vec![
                    ratatui::text::Span::styled(
                        slider.label.clone(),
                        if slider.is_selected {
                            Style::default()
                                .fg(Color::Blue)
                                .add_modifier(Modifier::BOLD)
                        } else {
                            Style::default().fg(Color::DarkGray)
                        },
                    ),
                ]))
                .borders(Borders::ALL)
                .border_type(Rounded)
                .border_style(if slider.is_selected {
                    Style::default()
                        .fg(Color::Blue)
                        .add_modifier(Modifier::BOLD)
                } else {
                    Style::default().fg(Color::DarkGray)
                })
                .render(slider_layout[index], frame.buffer_mut());
            Slider::from_state(&slider.state)
                .orientation(SliderOrientation::Vertical)
                .filled_symbol(slider.style.filled_symbol)
                .handle_symbol(slider.style.handle_symbol)
                .empty_symbol(slider.style.empty_symbol)
                .filled_color(if slider.is_selected {
                    Color::Blue
                } else {
                    Color::Gray
                })
                .handle_color(if slider.is_selected {
                    Color::Blue
                } else {
                    Color::Gray
                })
                .empty_color(Color::DarkGray)
                .show_value(true)
                .render(
                    slider_layout[index].inner(Margin::new(1, 2)),
                    frame.buffer_mut(),
                );
        });
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
