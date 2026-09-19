use ratatui::{
    Frame,
    layout::{Constraint, Direction, Layout, Offset, Rect, Size},
    widgets::{FrameExt, Widget},
};

use crate::ui::{
    CenterOpts, centered_rect, page_indicator, warning_msg,
    color_mixer::ColorMixerSection,
    help::HelpSection,
    image::ImageSection,
    pipeline::PipelineSection,
    preset::PresetSection,
    scope::ScopeSection,
    slider::SliderSection,
    wheel::WheelSection,
};

use super::{ActivePage, App, AppLayout, ASPECT_RATIOS, RESOLUTION, Status};

impl App {
    pub(super) fn draw(&self, frame: &mut Frame) {
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
                    PipelineSection::row_width(&self.pipeline),
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
                warning_msg(&format!(
                    "Terminal is too small for \ndisplay selected frame size\n\nTry change your resolution and aspect ratio\nwanted:(w:{}, h:{}), current:(w:{}, h:{})",
                    needed_width, needed_height, area.width, area.height
                )),
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

        let image_area = match self.layout {
            AppLayout::Horizontal => self.draw_horizontal(area, frame),
            AppLayout::Vertical => self.draw_vertical(area, image_size, frame),
        };

        let image_center_opts = CenterOpts {
            width: image_size.width.saturating_add(2),
            height: image_size.height.saturating_add(3),
            margin: 0,
        };

        let mut image_section =
            ImageSection::new(&self.image_handler, image_area, image_center_opts);
        image_section.aspect_ratio = ASPECT_RATIOS[self.selected_aspect_ratio_index];
        image_section.resolution = RESOLUTION[self.selected_resolution_index];
        image_section.export_status = self.active_export_status();
        image_section.render(area, frame.buffer_mut());

        let page_indicator = page_indicator(self.page, area.width);
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

    fn draw_horizontal(&self, area: Rect, frame: &mut Frame) -> Rect {
        let app_layout = Layout::default()
            .direction(Direction::Horizontal)
            .constraints([Constraint::Percentage(70), Constraint::Percentage(30)])
            .split(area);

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
                        height: PipelineSection::col_height(&self.pipeline),
                        margin: 0,
                    },
                    app_layout[1],
                );
                let pipeline_section = PipelineSection::new(
                    &self.pipeline,
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
            ActivePage::ColorMixer => {
                let curr_sliders = &self.color_mixer[self.selected_color_index].sliders;
                let color_mixer_section = ColorMixerSection::new(
                    &self.color_mixer,
                    self.layout,
                    self.selected_color_index,
                    curr_sliders.len() == self.selected_color_mixer_part_index,
                );
                let centered_area = centered_rect(
                    CenterOpts {
                        width: SliderSection::PANEL_WIDTH,
                        height: SliderSection::col_height(
                            &self.color_mixer[self.selected_color_index].sliders,
                        ) + 4,
                        margin: 0,
                    },
                    app_layout[1],
                );

                let slider_section = SliderSection::new(
                    curr_sliders,
                    self.selected_color_mixer_part_index,
                    self.layout,
                );
                slider_section
                    .render(centered_area.offset(Offset::new(0, 4)), frame.buffer_mut());
                color_mixer_section.render(
                    centered_area.centered_horizontally(Constraint::Length(21)),
                    frame.buffer_mut(),
                );
            }
        }

        app_layout[0]
    }

    fn draw_vertical(&self, area: Rect, image_size: Size, frame: &mut Frame) -> Rect {
        let image_area = Rect {
            x: area.x,
            y: area.y,
            width: area.width,
            height: image_size.height.saturating_add(3),
        };

        match self.page {
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
                    &self.pipeline,
                    self.selected_effect_index,
                    self.layout,
                );
                // there is n boxes and n+1 pipes
                let pipeline_area = centered_rect(
                    CenterOpts {
                        width: PipelineSection::row_width(&self.pipeline),
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
            ActivePage::ColorMixer => {
                let curr_sliders = &self.color_mixer[self.selected_color_index].sliders;
                let color_mixer = ColorMixerSection::new(
                    &self.color_mixer,
                    self.layout,
                    self.selected_color_index,
                    curr_sliders.len() == self.selected_color_mixer_part_index,
                );
                let all_area = centered_rect(
                    CenterOpts {
                        width: SliderSection::row_width(
                            &self.color_mixer[self.selected_color_index].sliders,
                        ) + 10,
                        height: SliderSection::PANEL_HEIGHT,
                        margin: 0,
                    },
                    Rect {
                        x: area.x,
                        y: image_area.bottom().saturating_add(1),
                        width: area.width,
                        height: SliderSection::PANEL_HEIGHT,
                    },
                );

                let slider_section = SliderSection::new(
                    curr_sliders,
                    self.selected_color_mixer_part_index,
                    self.layout,
                );
                slider_section.render(all_area.offset(Offset::new(8, 0)), frame.buffer_mut());
                color_mixer.render(
                    all_area.centered_vertically(Constraint::Length(10)),
                    frame.buffer_mut(),
                );
            }
        }

        image_area
    }

    fn active_preset_status(&self) -> Option<Status> {
        if let Some((ref status, ref time)) = self.preset_status
            && time.elapsed().as_secs() < 2
        {
            return Some(status.clone());
        }
        None
    }

    fn active_export_status(&self) -> Option<Status> {
        if let Some((ref status, ref time)) = self.export_status
            && time.elapsed().as_secs() < 3
        {
            return Some(status.clone());
        }
        None
    }
}
