mod input;

use ratatui::{
    DefaultTerminal, Frame,
    layout::{Constraint, Direction, Layout, Rect},
    widgets::{FrameExt, Widget},
};
use ratatui_explorer::{FileExplorer, FileExplorerBuilder};
use std::sync::mpsc;
use std::time::Instant;
use std::{io, path::PathBuf};

use crate::{
    image::{ColorGrade, ImageHandler},
    preset::PresetManager,
    ui::{
        CenterOpts, ExplorerType, centered_rect, explorer_theme,
        help::HelpSection,
        image::ImageSection,
        page_indicator,
        pipeline::{ColorEffects, PipelineSection},
        preset::PresetSection,
        scope::ScopeSection,
        slider::{SliderData, SliderSection, default_sliders},
        warning_msg,
        wheel::{WheelData, WheelSection, default_wheels},
    },
};

// I know these can be enum too but I dont know is it worth to implement .next() instead of using them as an array
const ASPECT_RATIOS: [(u8, u8); 5] = [(1, 1), (4, 3), (3, 4), (16, 9), (9, 16)];
const RESOLUTION: [u32; 4] = [240, 360, 480, 720];

#[derive(Copy, Clone, PartialEq)]
pub enum ActivePage {
    Sliders,
    Wheels,
    Scopes,
    Pipeline,
    Preset,
}

// I am not that happy about my horizontal layout I couldn't align things like I wanted
// The height of some components like wheels is to much I had to squish them to fit
// Also I now some help sections are not shows up in presets in horizontal layout if you size small enough
// I am settling for now maybe later I will be come with better layout
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AppLayout {
    Horizontal,
    Vertical,
}
#[derive(Debug, Clone, PartialEq)]
pub enum Status {
    Success(String),
    Error(String),
}

//TODO: Add theme section with nearest neighbor method map colors of current image to selected theme
//TODO: Add 3d LUT support
pub struct App {
    image_handler: ImageHandler,
    sliders: Vec<SliderData>,
    wheels: Vec<WheelData>,
    effects: Vec<ColorEffects>,
    file_explorer: FileExplorer,

    preset_explorer: FileExplorer,
    preset_dir: PathBuf,
    preset_input: String,
    is_preset_input_mode: bool,
    preset_status: Option<(Status, Instant)>,

    page: ActivePage,
    layout: AppLayout,

    selected_slider_index: usize,
    selected_wheel_index: usize,
    selected_effect_index: usize,
    selected_aspect_ratio_index: usize,
    selected_resolution_index: usize,

    is_help_view: bool,
    is_show_original: bool,
    is_original: bool,
    is_re_render: bool,
    is_file_explorer_visible: bool,
    is_image_selected: bool,
    is_proxy_enabled: bool,
    exit: bool,

    is_directory_selected: bool,
    export_rx: Option<mpsc::Receiver<Result<String, String>>>,
    export_status: Option<(Status, Instant)>,
}

impl App {
    pub fn new(initial_image: Option<PathBuf>) -> Self {
        let sliders = default_sliders();
        let wheels = default_wheels();
        let theme = explorer_theme(ExplorerType::File);
        let file_explorer = FileExplorerBuilder::build_with_theme(theme).unwrap();
        let preset_dir = PresetManager::create_presets_dir();
        let preset_explorer = Self::build_preset_explorer(preset_dir.clone());

        let mut app = App {
            image_handler: ImageHandler::new(),
            sliders,
            wheels,
            effects: ColorEffects::default(),
            file_explorer,

            preset_explorer,
            preset_dir,
            preset_input: String::new(),
            is_preset_input_mode: false,
            preset_status: None,

            page: ActivePage::Sliders,
            layout: AppLayout::Vertical,

            selected_slider_index: 0,
            selected_wheel_index: 0,
            selected_effect_index: 0,
            selected_aspect_ratio_index: 0,
            selected_resolution_index: 0,

            is_help_view: false,
            is_show_original: false,
            is_original: false,
            is_re_render: false,
            is_file_explorer_visible: false,
            is_directory_selected: false,
            is_image_selected: false,
            is_proxy_enabled: true,
            exit: false,

            export_rx: None,
            export_status: None,
        };

        if let Some(path) = initial_image {
            app.image_handler.load_from_path(path);
            app.is_re_render = true;
        }

        app
    }

    fn build_preset_explorer(preset_dir: PathBuf) -> FileExplorer {
        let theme = explorer_theme(ExplorerType::Preset);
        FileExplorerBuilder::default()
            .working_dir(preset_dir)
            .theme(theme)
            .filter_map(|file| {
                let keep = match file.path.extension() {
                    Some(ext) => ext.to_str().unwrap_or_default() == "toml",
                    None => file.is_dir,
                };
                if keep { Some(file) } else { None }
            })
            .build()
            .unwrap()
    }

    pub fn run(&mut self, terminal: &mut DefaultTerminal) -> io::Result<()> {
        while !self.exit {
            terminal.draw(|frame| self.draw(frame))?;
            self.handle_events()?;
            if self.is_image_selected {
                let path = self.file_explorer.current().path.clone();
                self.image_handler.load_from_path(path);
                self.is_re_render = true;
                self.is_image_selected = false;
            }
            if self.is_directory_selected {
                let path = self.file_explorer.current().path.clone();
                self.export_rx = Some(self.image_handler.save_to_path(path));
                self.export_status =
                    Some((Status::Success("Exporting...".to_string()), Instant::now()));
                self.is_directory_selected = false;
            }
            if let Some(ref rx) = self.export_rx {
                if let Ok(result) = rx.try_recv() {
                    self.export_status = Some((
                        match result {
                            Ok(name) => Status::Success(format!("Saved: {name}")),
                            Err(e) => Status::Error(e),
                        },
                        Instant::now(),
                    ));
                    self.export_rx = None;
                }
            }
            if self.is_re_render && self.image_handler.protocol.is_some() {
                if self.is_show_original && !self.is_original {
                    self.image_handler
                        .apply_effects(ColorGrade::default(), ColorEffects::default());
                    self.is_original = true;
                }
                if !self.is_show_original {
                    self.image_handler.apply_effects(
                        ColorGrade {
                            //sliders
                            temperature: self.sliders[0].state.value() as f32,
                            tint: self.sliders[1].state.value() as f32,
                            exposure: self.sliders[2].state.value() as f32,
                            contrast: self.sliders[3].state.value() as f32,
                            saturation: self.sliders[4].state.value() as f32,
                            hue_degrees: self.sliders[5].state.value() as f32,

                            //wheels
                            lift_x: self.wheels[0].x as f32,
                            lift_y: self.wheels[0].y as f32,
                            lift_lum: self.wheels[0].lum.state.value() as f32,
                            gamma_x: self.wheels[1].x as f32,
                            gamma_y: self.wheels[1].y as f32,
                            gamma_lum: self.wheels[1].lum.state.value() as f32,
                            gain_x: self.wheels[2].x as f32,
                            gain_y: self.wheels[2].y as f32,
                            gain_lum: self.wheels[2].lum.state.value() as f32,
                        },
                        self.effects.clone(),
                    );
                }
                self.is_re_render = false;
            }
            self.image_handler.poll();
        }
        Ok(())
    }

    fn draw(&self, frame: &mut Frame) {
        let area = frame.area();
        if self.is_file_explorer_visible {
            frame.render_widget_ref(self.file_explorer.widget(), area);
            return;
        }

        let needed_height;
        let needed_width;

        let image_size = self.image_handler.target_size;
        let too_small = match self.layout {
            AppLayout::Vertical => {
                let controls_height = [
                    SliderSection::PANEL_HEIGHT,
                    WheelSection::WHEEL_HEIGHT,
                    ScopeSection::SCOPE_HEIGHT,
                    PipelineSection::PIPELINE_HEIGHT,
                ];
                let controls_width = [
                    SliderSection::row_width(&self.sliders),
                    WheelSection::row_width(&self.wheels, self.layout),
                    ScopeSection::MIN_WIDTH,
                    PipelineSection::row_width(&self.effects),
                ];
                // image borders(2) + info line(1) + gap(1) + controls + page indicator(1)
                needed_height = image_size
                    .height
                    .saturating_add(3) // image borders + info line
                    .saturating_add(1) // gap between image and controls
                    .saturating_add(*controls_height.iter().max().unwrap())
                    .saturating_add(1) // page indicator
                    .max(HelpSection::HEIGHT); // check if help is taller
                needed_width = image_size
                    .width
                    .saturating_add(2)
                    .max(*controls_width.iter().max().unwrap());
                area.height < needed_height || area.width < needed_width
            }
            AppLayout::Horizontal => {
                // image borders(2) + left sidebar (sliders) + right sidebar (wheels)
                needed_width = image_size
                    .width
                    .saturating_add(2)
                    .saturating_add(SliderSection::PANEL_WIDTH)
                    .saturating_add(WheelSection::WHEEL_WIDTH);
                let slider_height = SliderSection::col_height(&self.sliders);
                let wheel_height = WheelSection::col_height(&self.wheels, self.layout);
                // max height stack + page indicator(1)
                needed_height = slider_height.max(wheel_height).saturating_add(1);
                area.height < needed_height || area.width < needed_width
            }
        };

        if too_small {
            frame.render_widget(
                warning_msg(
                    &format!(
                        "Terminal is too small for \ndisplay selected frame size\n\nTry change your resolution and aspect ratio\nwanted:(w:{}, h:{}), current:(w:{}, h:{})",
                        needed_width, needed_height, area.width, area.height
                    )
                ),
                centered_rect(
                    CenterOpts {
                        width: 50,
                        height: 5,
                        margin: 0,
                    },
                    area,
                ),
            );
            return;
        }

        if self.is_help_view {
            frame.render_widget(HelpSection {}, area);
            return;
        }

        let mut image_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: image_size.height.saturating_add(3), // Borders + space for text,
        };

        let image_center_opts = CenterOpts {
            width: image_size.width.saturating_add(2),   // For borders
            height: image_size.height.saturating_add(3), // Borders + space for text
            margin: 0,
        };

        match self.layout {
            AppLayout::Horizontal => {
                let app_layout = Layout::default()
                    .direction(Direction::Horizontal)
                    .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
                    .split(area);
                image_area = app_layout[0];
                match self.page {
                    ActivePage::Sliders => {
                        let slider_area = centered_rect(
                            CenterOpts {
                                width: SliderSection::PANEL_WIDTH,
                                height: SliderSection::col_height(&self.sliders),
                                margin: 0,
                            },
                            app_layout[1],
                        );
                        let slider_section = SliderSection::new(
                            &self.sliders,
                            self.selected_slider_index,
                            self.layout,
                        );
                        slider_section.render(slider_area, frame.buffer_mut());
                    }
                    ActivePage::Wheels => {
                        let wheel_area = centered_rect(
                            CenterOpts {
                                width: WheelSection::SMALL_WHEEL_WIDTH,
                                height: WheelSection::col_height(&self.wheels, self.layout),
                                margin: 0,
                            },
                            app_layout[1],
                        );
                        let wheel_section =
                            WheelSection::new(&self.wheels, self.selected_wheel_index, self.layout);
                        wheel_section.render(wheel_area, frame.buffer_mut());
                    }
                    ActivePage::Scopes => {
                        let scope_section =
                            ScopeSection::new(&self.image_handler.scope_data, self.layout);
                        scope_section.render(app_layout[1], frame.buffer_mut());
                    }
                    ActivePage::Pipeline => {
                        // there is n boxes and n+1 pipes
                        let pipeline_area = centered_rect(
                            CenterOpts {
                                width: PipelineSection::PIPELINE_WIDTH,
                                height: PipelineSection::col_height(&self.effects),
                                margin: 0,
                            },
                            app_layout[1],
                        );
                        let pipeline_section = PipelineSection::new(
                            &self.effects,
                            self.selected_effect_index,
                            self.layout,
                        );
                        pipeline_section.render(pipeline_area, frame.buffer_mut());
                    }
                    ActivePage::Preset => {
                        let (explorer_area, input_area) =
                            PresetSection::layout(app_layout[1], self.layout);
                        frame.render_widget_ref(self.preset_explorer.widget(), explorer_area);
                        let preset_section = PresetSection::new(
                            &self.preset_input,
                            self.is_preset_input_mode,
                            self.active_preset_status(),
                        );
                        preset_section.render(input_area, frame.buffer_mut());
                    }
                }
            }
            AppLayout::Vertical => match self.page {
                ActivePage::Sliders => {
                    let slider_area = centered_rect(
                        CenterOpts {
                            width: SliderSection::row_width(&self.sliders),
                            height: SliderSection::PANEL_HEIGHT,
                            margin: 0,
                        },
                        Rect::new(
                            area.x,
                            image_area.bottom().saturating_add(1),
                            area.width,
                            SliderSection::PANEL_HEIGHT,
                        ),
                    );

                    let slider_section =
                        SliderSection::new(&self.sliders, self.selected_slider_index, self.layout);
                    slider_section.render(slider_area, frame.buffer_mut());
                }
                ActivePage::Wheels => {
                    let wheel_area = centered_rect(
                        CenterOpts {
                            width: WheelSection::row_width(&self.wheels, self.layout),
                            height: WheelSection::WHEEL_HEIGHT,
                            margin: 0,
                        },
                        Rect::new(
                            area.x,
                            image_area.bottom().saturating_add(1),
                            area.width,
                            WheelSection::WHEEL_HEIGHT,
                        ),
                    );
                    let wheel_section =
                        WheelSection::new(&self.wheels, self.selected_wheel_index, self.layout);
                    wheel_section.render(wheel_area, frame.buffer_mut());
                }
                ActivePage::Scopes => {
                    let scope_section =
                        ScopeSection::new(&self.image_handler.scope_data, self.layout);
                    scope_section.render(
                        centered_rect(
                            CenterOpts {
                                width: if area.width < 100 {
                                    area.width.saturating_sub(10)
                                } else {
                                    area.width.saturating_sub(20)
                                },
                                height: ScopeSection::SCOPE_HEIGHT,
                                margin: 0,
                            },
                            Rect {
                                x: area.x,
                                y: image_area.bottom().saturating_add(1),
                                width: area.width,
                                height: ScopeSection::SCOPE_HEIGHT,
                            },
                        ),
                        frame.buffer_mut(),
                    );
                }
                ActivePage::Pipeline => {
                    let pipeline_section = PipelineSection::new(
                        &self.effects,
                        self.selected_effect_index,
                        self.layout,
                    );
                    // there is n boxes and n+1 pipes
                    let pipeline_area = centered_rect(
                        CenterOpts {
                            width: PipelineSection::row_width(&self.effects),
                            height: PipelineSection::PIPELINE_HEIGHT,
                            margin: 0,
                        },
                        Rect {
                            x: area.x,
                            y: image_area.bottom().saturating_add(1),
                            width: area.width,
                            height: PipelineSection::PIPELINE_HEIGHT,
                        },
                    );
                    pipeline_section.render(pipeline_area, frame.buffer_mut());
                }
                ActivePage::Preset => {
                    let preset_area = centered_rect(
                        CenterOpts {
                            width: if area.width < 100 {
                                area.width.saturating_sub(10)
                            } else {
                                area.width.saturating_sub(20)
                            },
                            height: PresetSection::EXPLORER_HEIGHT + PresetSection::INPUT_HEIGHT,
                            margin: 0,
                        },
                        Rect {
                            x: area.x,
                            y: image_area.bottom().saturating_add(1), // image/preset gap
                            width: area.width,
                            height: PresetSection::EXPLORER_HEIGHT + PresetSection::INPUT_HEIGHT,
                        },
                    );
                    let (explorer_area, input_area) =
                        PresetSection::layout(preset_area, self.layout);
                    frame.render_widget_ref(self.preset_explorer.widget(), explorer_area);
                    let preset_section = PresetSection::new(
                        &self.preset_input,
                        self.is_preset_input_mode,
                        self.active_preset_status(),
                    );
                    preset_section.render(input_area, frame.buffer_mut());
                }
            },
        }

        image_area = centered_rect(image_center_opts, image_area);
        let mut image_section = ImageSection::new(&self.image_handler, image_area);
        image_section.aspect_ratio = ASPECT_RATIOS[self.selected_aspect_ratio_index];
        image_section.resolution = RESOLUTION[self.selected_resolution_index];
        image_section.export_status = self.active_export_status();
        image_section.render(area, frame.buffer_mut());

        let page_indicator = page_indicator(self.page);
        page_indicator.render(
            Rect {
                x: area.x,
                y: area.bottom().saturating_sub(1),
                width: area.width,
                height: 1,
            },
            frame.buffer_mut(),
        );
    }

    fn active_preset_status(&self) -> Option<Status> {
        if let Some((ref status, ref time)) = self.preset_status {
            if time.elapsed().as_secs() < 2 {
                return Some(status.clone());
            }
        }
        None
    }

    fn active_export_status(&self) -> Option<Status> {
        if let Some((ref status, ref time)) = self.export_status {
            if time.elapsed().as_secs() < 3 {
                return Some(status.clone());
            }
        }
        None
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
