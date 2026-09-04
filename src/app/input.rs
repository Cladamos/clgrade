use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::io;

use crate::{
    input::{Action, map_key_to_action},
    ui::pipeline::ColorEffects,
};

use super::{ASPECT_RATIOS, App, RESOLUTION, SUPPORTED_FORMATS};

impl App {
    pub(super) fn handle_events(&mut self) -> io::Result<()> {
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
                self.page = super::ActivePage::Sliders;
            }
            Action::SwitchToWheels => {
                self.page = super::ActivePage::Wheels;
            }
            Action::SwitchToScopes => {
                self.page = super::ActivePage::Scopes;
            }
            Action::SwitchToPipeline => {
                self.page = super::ActivePage::Pipeline;
            }
            Action::ToggleLayout => {
                self.layout = match self.layout {
                    super::AppLayout::Horizontal => super::AppLayout::Vertical,
                    super::AppLayout::Vertical => super::AppLayout::Horizontal,
                };
            }
            Action::NextTool => match self.page {
                super::ActivePage::Sliders => {
                    self.selected_slider_index =
                        (self.selected_slider_index + 1) % self.sliders.len();
                }
                super::ActivePage::Wheels => {
                    let current_wheel = &mut self.wheels[self.selected_wheel_index];
                    if current_wheel.focused_part == crate::ui::wheel::SelectedPart::LumSlider {
                        current_wheel.focused_part = crate::ui::wheel::SelectedPart::Wheel;
                        self.selected_wheel_index =
                            (self.selected_wheel_index + 1) % self.wheels.len();
                    } else {
                        current_wheel.focused_part = crate::ui::wheel::SelectedPart::LumSlider;
                    }
                }
                super::ActivePage::Scopes => {}
                super::ActivePage::Pipeline => {
                    self.selected_effect_index =
                        (self.selected_effect_index + 1) % self.effects.len();
                }
            },
            Action::AdjustValue { delta_x, delta_y } => {
                match self.page {
                    super::ActivePage::Sliders => {
                        let s = &mut self.sliders[self.selected_slider_index];
                        let direction = delta_y + delta_x; // one is always 0
                        if direction > 0.0 {
                            s.state.increase(s.step);
                        } else {
                            s.state.decrease(s.step);
                        }
                    }
                    super::ActivePage::Wheels => {
                        let w = &mut self.wheels[self.selected_wheel_index];
                        if w.focused_part == crate::ui::wheel::SelectedPart::Wheel {
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
                    super::ActivePage::Scopes => {}
                    super::ActivePage::Pipeline => {
                        let p = &mut self.effects;
                        let len = p.len();
                        let i = self.selected_effect_index;
                        let direction = if self.layout == super::AppLayout::Horizontal {
                            // In horizontal view, y-axis is inverted for up/down
                            -delta_y + delta_x
                        } else {
                            delta_x + delta_y
                        };
                        if direction > 0.0 {
                            let next_i = (i + 1) % len;
                            p.swap(i, next_i);
                            self.selected_effect_index = next_i;
                        } else {
                            let next_i = (i + len - 1) % len;
                            p.swap(i, next_i);
                            self.selected_effect_index = next_i;
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
                super::ActivePage::Sliders => {
                    let s = &mut self.sliders[self.selected_slider_index];
                    s.state.set_value(s.default_value);
                    self.is_re_render = true;
                }
                super::ActivePage::Wheels => {
                    let w = &mut self.wheels[self.selected_wheel_index];
                    if w.focused_part == crate::ui::wheel::SelectedPart::Wheel {
                        w.x = 0.0;
                        w.y = 0.0;
                    } else {
                        w.lum.state.set_value(w.lum.default_value);
                    }
                    self.is_re_render = true;
                }
                super::ActivePage::Scopes => {}
                super::ActivePage::Pipeline => {
                    self.effects = ColorEffects::default();
                    self.selected_effect_index = 0;
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
                self.effects = ColorEffects::default();
                self.selected_effect_index = 0;
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
}
