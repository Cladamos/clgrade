use crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use ratatui::{
    DefaultTerminal, Frame,
    crossterm::{self},
    layout::Rect,
    widgets::{FrameExt, Widget},
};
use ratatui_explorer::{FileExplorer, FileExplorerBuilder};
use std::io;

use crate::{
    image::{ColorGrade, ImageHandler},
    input::{Action, map_key_to_action},
    ui::{
        CenterOpts, centered_rect, file_explorer_theme,
        image::ImageSection,
        slider::{SliderData, SliderSection, default_sliders},
        warning_msg,
        wheel::{SelectedPart, WheelData, WheelSection, default_wheels},
    },
};

const SUPPORTED_FORMATS: &[&str] = &["png", "jpg", "jpeg", "webp"];

// I know these can be enum too but I dont know is it worth to implement .next() instead of using them as an array
const ASPECT_RATIOS: [(u8, u8); 5] = [(1, 1), (4, 3), (3, 4), (16, 9), (9, 16)];
const RESOLUTION: [u32; 4] = [240, 360, 480, 720];

pub enum ActivePage {
    Sliders,
    Wheels,
}

pub struct App {
    image_handler: ImageHandler,
    sliders: Vec<SliderData>,
    wheels: Vec<WheelData>,
    file_explorer: FileExplorer,

    page: ActivePage,
    selected_slider_index: usize,
    selected_wheel_index: usize,
    selected_aspect_ratio_index: usize,
    selected_resolution_index: usize,

    is_show_original: bool,
    is_original: bool,
    is_re_render: bool,
    is_file_explorer_visible: bool,
    is_directory_selected: bool,
    is_image_selected: bool,
    is_proxy_enabled: bool,
    exit: bool,
}

impl App {
    pub fn new() -> Self {
        let sliders = default_sliders();
        let wheels = default_wheels();
        let theme = file_explorer_theme();
        let file_explorer = FileExplorerBuilder::build_with_theme(theme).unwrap();

        App {
            image_handler: ImageHandler::new(),
            sliders,
            wheels,
            file_explorer,

            page: ActivePage::Sliders,
            selected_slider_index: 0,
            selected_wheel_index: 0,
            selected_aspect_ratio_index: 0,
            selected_resolution_index: 1,

            is_show_original: false,
            is_original: false,
            is_re_render: false,
            is_file_explorer_visible: false,
            is_directory_selected: false,
            is_image_selected: false,
            is_proxy_enabled: true,
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
            if self.is_directory_selected {
                let path = self.file_explorer.current().path.clone();
                self.image_handler.save_to_path(path);
                self.is_directory_selected = false;
            }
            if self.is_re_render && self.image_handler.protocol.is_some() {
                if self.is_show_original && !self.is_original {
                    self.image_handler.apply_effects(ColorGrade::default());
                    self.is_original = true;
                }
                if !self.is_show_original {
                    self.image_handler.apply_effects(ColorGrade {
                        //sliders
                        temperature: self.sliders[0].state.value() as f32,
                        tint: self.sliders[1].state.value() as f32,
                        exposure: self.sliders[2].state.value() as f32,
                        contrast: self.sliders[3].state.value() as f32,
                        saturation: self.sliders[4].state.value() as f32,
                        hue_degrees: self.sliders[5].state.value() as f32,

                        //wheels
                        lift_x: self.wheels[0].x,
                        lift_y: self.wheels[0].y,
                        lift_lum: self.wheels[0].lum.state.value() as f32,
                        gamma_x: self.wheels[1].x,
                        gamma_y: self.wheels[1].y,
                        gamma_lum: self.wheels[1].lum.state.value() as f32,
                        gain_x: self.wheels[2].x,
                        gain_y: self.wheels[2].y,
                        gain_lum: self.wheels[2].lum.state.value() as f32,
                    });
                }
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
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press ||  key_event.kind == KeyEventKind::Repeat => {
                    self.handle_key_event(key_event)
                }
                Event::Key(key_event)
                    // I have not any other events to have release action so I am checking the space bar over here
                    // I can write the space bar release action in here,
                    // but inside of handle_key_event function it look better
                    if key_event.kind == KeyEventKind::Release && key_event.code == KeyCode::Char(' ') => {
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
        match map_key_to_action(key_event) {
            Action::ExportImage => {
                self.file_explorer
                    .set_filter_map(|file| if file.is_dir { Some(file) } else { None })
                    .unwrap();
                self.is_file_explorer_visible = true;
            }
            Action::Quit => self.exit(),
            Action::ToggleFileExplorer => {
                self.file_explorer
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
                self.is_file_explorer_visible = !self.is_file_explorer_visible
            }
            Action::Save => {
                if self.is_file_explorer_visible
                    && self.file_explorer.current().is_dir
                    && self.image_handler.protocol.is_some()
                {
                    self.is_directory_selected = true;
                    self.is_file_explorer_visible = false;
                }
            }
            Action::Select => {
                if self.is_file_explorer_visible && !self.file_explorer.current().is_dir {
                    self.is_image_selected = true;
                    self.is_file_explorer_visible = false;
                }
            }
            Action::SwitchToSliders => {
                self.page = ActivePage::Sliders;
            }
            Action::SwitchToWheels => {
                self.page = ActivePage::Wheels;
            }
            Action::NextTool => match self.page {
                ActivePage::Sliders => {
                    self.selected_slider_index =
                        (self.selected_slider_index + 1) % self.sliders.len();
                }
                ActivePage::Wheels => {
                    let current_wheel = &mut self.wheels[self.selected_wheel_index];
                    if current_wheel.focused_part == SelectedPart::LumSlider {
                        current_wheel.focused_part = SelectedPart::Wheel;
                        self.selected_wheel_index =
                            (self.selected_wheel_index + 1) % self.wheels.len();
                    } else {
                        current_wheel.focused_part = SelectedPart::LumSlider;
                    }
                }
            },
            Action::AdjustValue { delta_x, delta_y } => {
                match self.page {
                    ActivePage::Sliders => {
                        let s = &mut self.sliders[self.selected_slider_index];
                        let direction = delta_y + delta_x; // one is always 0
                        if direction > 0.0 {
                            s.state.increase(s.step);
                        } else {
                            s.state.decrease(s.step);
                        }
                    }
                    ActivePage::Wheels => {
                        let w = &mut self.wheels[self.selected_wheel_index];
                        if w.focused_part == SelectedPart::Wheel {
                            let step = 0.05;
                            let new_x = w.x + delta_x * step;
                            let new_y = w.y + delta_y * step;
                            if (-1.0..=1.0).contains(&new_x) {
                                w.x = new_x;
                            }
                            if (-1.0..=1.0).contains(&new_y) {
                                w.y = new_y;
                            }
                        } else {
                            let s = &mut w.lum;
                            let direction = delta_y + delta_x;
                            if direction > 0.0 {
                                s.state.increase(s.step);
                            } else {
                                s.state.decrease(s.step);
                            }
                        }
                    }
                }
                self.is_re_render = true;
            }
            Action::ChangeAspectRatio => {
                self.selected_aspect_ratio_index =
                    (self.selected_aspect_ratio_index + 1) % ASPECT_RATIOS.len();
                self.image_handler.set_resolution(
                    RESOLUTION[self.selected_resolution_index],
                    ASPECT_RATIOS[self.selected_aspect_ratio_index],
                );
                self.image_handler.reload();
                self.is_re_render = true;
            }
            Action::ChangeResolution => {
                self.selected_resolution_index =
                    (self.selected_resolution_index + 1) % RESOLUTION.len();
                self.image_handler.set_resolution(
                    RESOLUTION[self.selected_resolution_index],
                    ASPECT_RATIOS[self.selected_aspect_ratio_index],
                );
                self.image_handler.reload();
                self.is_re_render = true;
            }
            Action::ResetTool => match self.page {
                ActivePage::Sliders => {
                    let s = &mut self.sliders[self.selected_slider_index];
                    s.state.set_value(s.default_value);
                    self.is_re_render = true;
                }
                ActivePage::Wheels => {
                    let w = &mut self.wheels[self.selected_wheel_index];
                    if w.focused_part == SelectedPart::Wheel {
                        w.x = 0.0;
                        w.y = 0.0;
                    } else {
                        w.lum.state.set_value(w.lum.default_value);
                    }
                    self.is_re_render = true;
                }
            },
            Action::ResetAll => {
                self.sliders
                    .iter_mut()
                    .for_each(|s| s.state.set_value(s.default_value));
                self.wheels.iter_mut().for_each(|w| {
                    w.x = 0.0;
                    w.y = 0.0;
                    w.lum.state.set_value(w.lum.default_value);
                });
                self.is_re_render = true;
            }
            Action::ToggleOriginal => {
                if key_event.kind == KeyEventKind::Press {
                    self.is_show_original = true;
                    self.is_re_render = true;
                }
                if key_event.kind == KeyEventKind::Release {
                    self.is_show_original = false;
                    self.is_original = false;
                    self.is_re_render = true;
                }
            }
            Action::ToggleProxy => {
                self.is_proxy_enabled = !self.is_proxy_enabled;
                self.image_handler.is_proxy_enabled = self.is_proxy_enabled;
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

        let image_size = self.image_handler.target_size;
        // TODO: 9:16 720p is too big find a solution I think horizontal layout will be great
        if image_size.height + 19 > area.height {
            frame.render_widget(
                warning_msg("Terminal is too small for \ndisplay selected frame size\n\nTry change your resolution and aspect ratio."),
                centered_rect(
                    CenterOpts {
                        width: 50,
                        height: 4,
                        margin: 0,
                    },
                    area,
                ),
            );
            return;
        }

        let image_area = centered_rect(
            CenterOpts {
                width: image_size.width.saturating_add(2),   // For borders
                height: image_size.height.saturating_add(3), // Borders + space for text
                margin: 0,
            },
            Rect {
                x: area.x,
                y: area.y,
                width: area.width,
                height: image_size.height.saturating_add(3), // Borders + space for text,
            },
        );

        match self.page {
            ActivePage::Sliders => {
                let slider_height = area.height - image_area.bottom().saturating_sub(2);
                let slider_area = centered_rect(
                    CenterOpts {
                        //TODO: slider width is constant if you change in ui/slider.rs you need to change here too. fix it
                        width: (self.sliders.len() * 13) as u16,
                        height: slider_height,
                        margin: 0,
                    },
                    Rect::new(
                        area.x,
                        image_area.bottom().saturating_add(1),
                        area.width,
                        slider_height,
                    ),
                );
                let slider_section = SliderSection::new(&self.sliders, self.selected_slider_index);
                slider_section.render(slider_area, frame.buffer_mut());
            }
            ActivePage::Wheels => {
                let wheel_height = 16;
                let wheel_area = centered_rect(
                    CenterOpts {
                        //TODO: wheel width is constants if you change in ui/wheel.rs you need to change here too. fix it
                        width: self.wheels.len() as u16 * 21,
                        height: wheel_height,
                        margin: 0,
                    },
                    Rect::new(
                        area.x,
                        image_area.bottom().saturating_add(1),
                        area.width,
                        wheel_height,
                    ),
                );
                let wheel_section = WheelSection::new(&self.wheels, self.selected_wheel_index);
                wheel_section.render(wheel_area, frame.buffer_mut());
            }
        }

        let mut image_section = ImageSection::new(&self.image_handler);
        image_section.aspect_ratio = ASPECT_RATIOS[self.selected_aspect_ratio_index];
        image_section.resolution = RESOLUTION[self.selected_resolution_index];
        image_section.render(image_area, frame.buffer_mut());
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
