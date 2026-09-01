mod input;

use ratatui::{
    DefaultTerminal, Frame,
    layout::Rect,
    widgets::{FrameExt, Widget},
};
use ratatui_explorer::{FileExplorer, FileExplorerBuilder};
use std::io;

use crate::{
    image::{ColorGrade, ImageHandler},
    ui::{
        CenterOpts, centered_rect, file_explorer_theme,
        image::ImageSection,
        page_indicator,
        pipeline::{ColorEffects, PipelineSection},
        scope::ScopeSection,
        slider::{SliderData, SliderSection, default_sliders},
        warning_msg,
        wheel::{WheelData, WheelSection, default_wheels},
    },
};

const SUPPORTED_FORMATS: &[&str] = &["png", "jpg", "jpeg", "webp"];

// I know these can be enum too but I dont know is it worth to implement .next() instead of using them as an array
const ASPECT_RATIOS: [(u8, u8); 5] = [(1, 1), (4, 3), (3, 4), (16, 9), (9, 16)];
const RESOLUTION: [u32; 4] = [240, 360, 480, 720];

#[derive(Copy, Clone)]
pub enum ActivePage {
    Sliders,
    Wheels,
    Scopes,
    Pipeline,
}
#[derive(Debug, Copy, Clone, PartialEq)]
pub enum AppLayout {
    Horizontal,
    Vertical,
}

pub struct App {
    image_handler: ImageHandler,
    sliders: Vec<SliderData>,
    wheels: Vec<WheelData>,
    effects: Vec<ColorEffects>,
    file_explorer: FileExplorer,

    page: ActivePage,
    layout: AppLayout,

    selected_slider_index: usize,
    selected_wheel_index: usize,
    selected_effect_index: usize,
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
            effects: ColorEffects::default(),
            file_explorer,

            page: ActivePage::Sliders,
            layout: AppLayout::Vertical,

            selected_slider_index: 0,
            selected_wheel_index: 0,
            selected_effect_index: 0,
            selected_aspect_ratio_index: 0,
            selected_resolution_index: 0,

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
                            lift_x: self.wheels[0].x,
                            lift_y: self.wheels[0].y,
                            lift_lum: self.wheels[0].lum.state.value() as f32,
                            gamma_x: self.wheels[1].x,
                            gamma_y: self.wheels[1].y,
                            gamma_lum: self.wheels[1].lum.state.value() as f32,
                            gain_x: self.wheels[2].x,
                            gain_y: self.wheels[2].y,
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

        let image_size = self.image_handler.target_size;
        let too_small = match self.layout {
            AppLayout::Vertical => {
                let controls_height = match self.page {
                    ActivePage::Sliders => SliderSection::PANEL_HEIGHT,
                    ActivePage::Wheels => WheelSection::WIDGET_HEIGHT,
                    ActivePage::Scopes => ScopeSection::SCOPE_HEIGHT,
                    ActivePage::Pipeline => PipelineSection::PIPELINE_HEIGHT,
                };
                let controls_width = match self.page {
                    ActivePage::Sliders => self.sliders.len() as u16 * SliderSection::ITEM_WIDTH,
                    ActivePage::Wheels => self.wheels.len() as u16 * WheelSection::WIDGET_WIDTH,
                    ActivePage::Scopes => ScopeSection::SCOPE_HEIGHT,
                    ActivePage::Pipeline => PipelineSection::PIPELINE_HEIGHT,
                };
                // image borders(2) + info line(1) + gap(1) + controls + page indicator(1)
                let needed_height = image_size
                    .height
                    .saturating_add(3) // image borders + info line
                    .saturating_add(1) // gap between image and controls
                    .saturating_add(controls_height)
                    .saturating_add(1); // page indicator
                let needed_width = image_size.width.saturating_add(2).max(controls_width);
                area.height < needed_height || area.width < needed_width
            }
            AppLayout::Horizontal => {
                // image borders(2) + left sidebar (sliders) + right sidebar (wheels)
                let needed_width = image_size
                    .width
                    .saturating_add(2)
                    .saturating_add(SliderSection::PANEL_WIDTH)
                    .saturating_add(WheelSection::WIDGET_WIDTH);
                let slider_stack = self.sliders.len() as u16 * SliderSection::ITEM_HEIGHT;
                let wheel_stack = self.wheels.len() as u16 * WheelSection::WIDGET_HEIGHT;
                // max height stack + page indicator(1)
                let needed_height = slider_stack.max(wheel_stack).saturating_add(1);
                area.height < needed_height || area.width < needed_width
            }
        };

        if too_small {
            frame.render_widget(
                warning_msg(
                    "Terminal is too small for \ndisplay selected frame size\n\nTry change your resolution and aspect ratio.",
                ),
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

        match self.layout {
            AppLayout::Horizontal => {
                let slider_stack_height = self.sliders.len() as u16 * SliderSection::ITEM_HEIGHT;
                let slider_area = centered_rect(
                    CenterOpts {
                        width: SliderSection::PANEL_WIDTH,
                        height: slider_stack_height,
                        margin: 0,
                    },
                    Rect::new(
                        area.x,
                        area.y,
                        SliderSection::PANEL_WIDTH,
                        slider_stack_height,
                    ),
                );
                let slider_section =
                    SliderSection::new(&self.sliders, self.selected_slider_index, self.layout);
                slider_section.render(slider_area, frame.buffer_mut());

                let wheel_stack_height = self.wheels.len() as u16 * WheelSection::WIDGET_HEIGHT;
                let wheel_area = centered_rect(
                    CenterOpts {
                        width: WheelSection::WIDGET_WIDTH,
                        height: wheel_stack_height,
                        margin: 0,
                    },
                    Rect::new(
                        area.right().saturating_sub(WheelSection::WIDGET_WIDTH),
                        area.y,
                        WheelSection::WIDGET_WIDTH,
                        wheel_stack_height,
                    ),
                );
                let wheel_section =
                    WheelSection::new(&self.wheels, self.selected_wheel_index, self.layout);
                wheel_section.render(wheel_area, frame.buffer_mut());
            }
            AppLayout::Vertical => match self.page {
                ActivePage::Sliders => {
                    let slider_area = centered_rect(
                        CenterOpts {
                            width: self.sliders.len() as u16 * SliderSection::ITEM_WIDTH,
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
                            width: self.wheels.len() as u16 * WheelSection::WIDGET_WIDTH,
                            height: WheelSection::WIDGET_HEIGHT,
                            margin: 0,
                        },
                        Rect::new(
                            area.x,
                            image_area.bottom().saturating_add(1),
                            area.width,
                            WheelSection::WIDGET_HEIGHT,
                        ),
                    );
                    let wheel_section =
                        WheelSection::new(&self.wheels, self.selected_wheel_index, self.layout);
                    wheel_section.render(wheel_area, frame.buffer_mut());
                }
                ActivePage::Scopes => {
                    let scope_section = ScopeSection::new(&self.image_handler.scope_data);
                    scope_section.render(
                        centered_rect(
                            CenterOpts {
                                width: (area.width / 3) * 2,
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
                    let pipeline_section =
                        PipelineSection::new(&self.effects, self.selected_effect_index);
                    // there is n boxes and n+1 pipes
                    let pipeline_area = centered_rect(
                        CenterOpts {
                            width: self.effects.len() as u16
                                * (PipelineSection::BOX_WIDTH + PipelineSection::PIPE_WIDTH)
                                + PipelineSection::PIPE_WIDTH,
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
            },
        }

        let mut image_section = ImageSection::new(&self.image_handler);
        image_section.aspect_ratio = ASPECT_RATIOS[self.selected_aspect_ratio_index];
        image_section.resolution = RESOLUTION[self.selected_resolution_index];
        image_section.render(image_area, frame.buffer_mut());

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

    fn exit(&mut self) {
        self.exit = true;
    }
}
