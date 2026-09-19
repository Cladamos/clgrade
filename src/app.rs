mod draw;
mod history;
mod input;

use ratatui::DefaultTerminal;
use ratatui_explorer::{FileExplorer, FileExplorerBuilder};
use std::sync::mpsc;
use std::time::Instant;
use std::{io, path::PathBuf};

use crate::{
    app::history::{History, Snapshot},
    effect::{SliderDatas, WheelDatas},
    image::ImageHandler,
    preset::PresetManager,
    ui::{
        ExplorerType,
        color_mixer::ColorMixerPart,
        explorer_theme,
        pipeline::ColorEffects,
        slider::{SliderData, default_sliders},
        wheel::{WheelData, default_wheels},
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
    ColorMixer,
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
    pipeline: Vec<ColorEffects>,
    color_mixer: Vec<ColorMixerPart>,
    file_explorer: FileExplorer,
    history: History,

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
    selected_color_index: usize,
    selected_color_mixer_part_index: usize,
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
        let color_mixer = ColorMixerPart::default_parts();
        let theme = explorer_theme(ExplorerType::File);
        let file_explorer = FileExplorerBuilder::build_with_theme(theme).unwrap();
        let preset_dir = PresetManager::create_presets_dir();
        let preset_explorer = Self::build_preset_explorer(preset_dir.clone());

        let mut app = App {
            image_handler: ImageHandler::new(),
            sliders,
            wheels,
            pipeline: ColorEffects::default_pipeline(),
            color_mixer,
            file_explorer,
            history: History::new(),

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
            selected_color_index: 0,
            selected_color_mixer_part_index: 3,
            selected_aspect_ratio_index: 0,
            selected_resolution_index: 0,

            is_help_view: false,
            is_show_original: false,
            is_original: false,
            is_re_render: false,
            is_file_explorer_visible: false,
            is_directory_selected: false,
            is_image_selected: false,
            is_proxy_enabled: false,
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
            if let Some(ref rx) = self.export_rx
                && let Ok(result) = rx.try_recv()
            {
                self.export_status = Some((
                    match result {
                        Ok(name) => Status::Success(format!("Saved: {name}")),
                        Err(e) => Status::Error(e),
                    },
                    Instant::now(),
                ));
                self.export_rx = None;
            }
            if self.is_re_render && self.image_handler.protocol.is_some() {
                if self.is_show_original && !self.is_original {
                    self.image_handler.apply_effects(
                        SliderDatas::default(),
                        WheelDatas::default(),
                        ColorEffects::default_pipeline(),
                        ColorMixerPart::default_parts(),
                    );
                    self.is_original = true;
                }
                if !self.is_show_original {
                    self.image_handler.apply_effects(
                        self.get_slider_datas(),
                        self.get_wheel_datas(),
                        self.pipeline.clone(),
                        self.color_mixer.clone(),
                    );
                }
                self.is_re_render = false;
            }
            self.image_handler.poll();
        }
        Ok(())
    }
    fn get_slider_datas(&self) -> SliderDatas {
        SliderDatas {
            temperature: self.sliders[0].state.value() as f32,
            tint: self.sliders[1].state.value() as f32,
            exposure: self.sliders[2].state.value() as f32,
            contrast: self.sliders[3].state.value() as f32,
            saturation: self.sliders[4].state.value() as f32,
            hue_degrees: self.sliders[5].state.value() as f32,
        }
    }
    fn get_wheel_datas(&self) -> WheelDatas {
        WheelDatas {
            lift_x: self.wheels[0].x as f32,
            lift_y: self.wheels[0].y as f32,
            lift_lum: self.wheels[0].lum.state.value() as f32,
            gamma_x: self.wheels[1].x as f32,
            gamma_y: self.wheels[1].y as f32,
            gamma_lum: self.wheels[1].lum.state.value() as f32,
            gain_x: self.wheels[2].x as f32,
            gain_y: self.wheels[2].y as f32,
            gain_lum: self.wheels[2].lum.state.value() as f32,
        }
    }
    fn get_snapshot(&self) -> Snapshot {
        Snapshot {
            slider_datas: self.get_slider_datas(),
            wheel_datas: self.get_wheel_datas(),
            pipeline: self.pipeline.clone(),
            color_mixer: self.color_mixer.clone(),
        }
    }

    fn apply_snapshot(&mut self, snapshot: Snapshot) {
        self.pipeline = snapshot.pipeline;

        let slider_values = snapshot.slider_datas.to_array();
        self.sliders
            .iter_mut()
            .enumerate()
            .for_each(|(i, s)| s.state.set_value(slider_values[i] as f64));

        let wheel_values = snapshot.wheel_datas.to_array();
        self.wheels.iter_mut().enumerate().for_each(|(i, w)| {
            w.x = wheel_values[i][0] as f64;
            w.y = wheel_values[i][1] as f64;
            w.lum.state.set_value(wheel_values[i][2] as f64);
        });

        self.color_mixer = snapshot.color_mixer;
    }

    fn exit(&mut self) {
        self.exit = true;
    }
}
