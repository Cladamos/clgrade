use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame, crossterm,
    layout::Rect,
    text::Text,
    widgets::{FrameExt, Widget},
};
use ratatui_explorer::{FileExplorer, FileExplorerBuilder, Theme};
use std::io;
use tui_slider::{SliderState, style::SliderStyle};

use crate::{
    image::{ColorGrade, ImageHandler},
    ui::{CenterOpts, centered_rect, image::ImageSection, slider::SliderSection},
};

// I decided remove 720p so the protocol takes lots of time
// I would like to see it but the applying effects only takes ~5ms and the procotol takes ~330ms
// Because of transfering to raw image then base64 and pushing them into the terminal output buffer
// So the total time is around 335ms for the image to show up
// I am not sure if there is a way around that :(

const ASPECT_RATIOS: [(u8, u8); 5] = [(1, 1), (4, 3), (3, 4), (16, 9), (9, 16)];
const RESOLUTION: [u32; 3] = [240, 360, 480];

pub struct App {
    image_handler: ImageHandler,
    sliders: Vec<SliderData>,
    selected_slider_index: usize,
    selected_aspect_ratio_index: usize,
    selected_resolution_index: usize,
    is_re_render: bool,
    is_file_explorer_visible: bool,
    is_image_selected: bool,
    file_explorer: FileExplorer,
    exit: bool,
}

pub struct SliderData {
    pub label: &'static str,
    pub state: SliderState,
    pub step: f64,
    pub slider_style: SliderStyle,
}

const SUPPORTED_FORMATS: &[&str] = &["png", "jpg", "jpeg", "webp"];

impl App {
    pub fn new() -> Self {
        let sliders: Vec<SliderData> = vec![
            SliderData {
                label: "Temp",
                state: SliderState::new(0.0, -100.0, 100.0),
                step: 1.0,
                slider_style: SliderStyle::vertical(),
            },
            SliderData {
                label: "Exp",
                state: SliderState::new(0.0, -3.0, 3.0),
                step: 0.05,
                slider_style: SliderStyle::vertical(),
            },
            SliderData {
                label: "Cont",
                state: SliderState::new(0.0, -100.0, 100.0),
                step: 1.0,
                slider_style: SliderStyle::vertical(),
            },
            SliderData {
                label: "Sat",
                state: SliderState::new(1.0, 0.0, 2.0),
                step: 0.01,
                slider_style: SliderStyle::vertical(),
            },
            SliderData {
                label: "Hue",
                state: SliderState::new(0.0, -180.0, 180.0),
                step: 2.0,
                slider_style: SliderStyle::vertical(),
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
            selected_aspect_ratio_index: 0,
            selected_resolution_index: 1,
            is_re_render: false,
            is_file_explorer_visible: false,
            is_image_selected: false,
            file_explorer,
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
            if self.is_re_render && self.image_handler.protocol.is_some() {
                self.image_handler.apply_effects(ColorGrade {
                    temperature: self.sliders[0].state.value() as f32,
                    exposure: self.sliders[1].state.value() as f32,
                    contrast: self.sliders[2].state.value() as f32,
                    saturation: self.sliders[3].state.value() as f32,
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
                if self.is_file_explorer_visible && !self.file_explorer.current().is_dir {
                    self.is_image_selected = true;
                    self.is_file_explorer_visible = false;
                }
            }
            KeyCode::Tab => {
                self.selected_slider_index = (self.selected_slider_index + 1) % self.sliders.len();
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
            KeyCode::Char('a') => {
                self.selected_aspect_ratio_index =
                    (self.selected_aspect_ratio_index + 1) % ASPECT_RATIOS.len();
                self.image_handler.set_resolution(
                    RESOLUTION[self.selected_resolution_index],
                    ASPECT_RATIOS[self.selected_aspect_ratio_index],
                );
                self.image_handler.reload();
                self.is_re_render = true;
            }
            KeyCode::Char('A') => {
                self.selected_resolution_index =
                    (self.selected_resolution_index + 1) % RESOLUTION.len();
                self.image_handler.set_resolution(
                    RESOLUTION[self.selected_resolution_index],
                    ASPECT_RATIOS[self.selected_aspect_ratio_index],
                );
                self.image_handler.reload();
                self.is_re_render = true;
            }

            _ => {}
        }
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        if self.is_file_explorer_visible {
            frame.render_widget_ref(self.file_explorer.widget(), area);
            return;
        }
        if area.height < 33 {
            frame.render_widget(Text::from("Terminal is too small"), area);
            return;
        }

        let image_size = self.image_handler.target_size;

        let image_area = centered_rect(
            CenterOpts {
                width: image_size.width + 2,   // For borders
                height: image_size.height + 3, // Borders + space for text
                margin: 0,
            },
            Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: image_size.height + 3, // Borders + space for text,
            },
        );

        let slider_height = area.height - image_area.bottom() - 2;
        let slider_area = centered_rect(
            CenterOpts {
                //TODO: slider width with constant if you change in ui/slider.rs you need to change here too. fix it
                width: (self.sliders.len() * 15) as u16,
                height: slider_height,
                margin: 0,
            },
            Rect::new(area.x, image_area.bottom() + 1, area.width, slider_height),
        );
        let slider_section = SliderSection::new(&self.sliders, self.selected_slider_index);
        slider_section.render(slider_area, frame.buffer_mut());

        let mut image_section = ImageSection::new(&self.image_handler);
        image_section.aspect_ratio = ASPECT_RATIOS[self.selected_aspect_ratio_index];
        image_section.resolution = RESOLUTION[self.selected_resolution_index];
        image_section.render(image_area, frame.buffer_mut());
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
