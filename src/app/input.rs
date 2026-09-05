use ratatui::crossterm::event::{self, Event, KeyCode, KeyEvent, KeyEventKind};
use std::io;
use std::time::Instant;

use crate::{
    app::PresetStatus,
    input::{
        Action::{self},
        map_key_to_action,
    },
    preset::PresetManager,
    ui::pipeline::ColorEffects,
};

use super::{ASPECT_RATIOS, ActivePage, App, AppLayout, RESOLUTION, SUPPORTED_FORMATS};

impl App {
    pub(super) fn handle_events(&mut self) -> io::Result<()> {
        if event::poll(std::time::Duration::from_millis(16))? {
            let event = event::read()?;

            if self.is_preset_input_mode {
                match event {
                    Event::Key(key_event)
                        if key_event.kind == KeyEventKind::Press
                            || key_event.kind == KeyEventKind::Repeat =>
                    {
                        self.handle_preset_input_mode_event(key_event);
                        return Ok(());
                    }
                    _ => {}
                }
            }
            // I solved same keyevents handling multiple times with disable_next_events
            // I don't know it is the best way to solve it but it is working for now
            let disable_next_events = match event {
                Event::Key(key_event) if key_event.kind == KeyEventKind::Press || key_event.kind == KeyEventKind::Repeat => {
                    self.handle_key_event(key_event)
                }
                Event::Key(key_event)
                // I have not any other events to have release action so I am checking the space bar over here
                // I can write the space bar release action in here,
                // but inside of handle_key_event function it look better
                if key_event.kind == KeyEventKind::Release && key_event.code == KeyCode::Char(' ') => {
                    self.handle_key_event(key_event)
                }
                _ => false,
            };

            if disable_next_events {
                return Ok(());
            }

            if self.is_file_explorer_visible {
                self.file_explorer.handle(&event)?;
                if self.page == ActivePage::Preset {
                    return Ok(());
                }
            }

            if self.page == ActivePage::Preset {
                self.preset_explorer.handle(&event)?;
            }
        }

        Ok(())
    }
    fn handle_preset_input_mode_event(&mut self, key_event: KeyEvent) {
        match key_event.code {
            KeyCode::Char(c) => {
                self.preset_input.push(c);
            }
            KeyCode::Backspace => {
                self.preset_input.pop();
            }
            KeyCode::Enter => {
                if !self.preset_input.is_empty() {
                    let data =
                        PresetManager::from_app_state(&self.sliders, &self.wheels, &self.effects);
                    match PresetManager::save(&self.preset_input, &data, self.preset_dir.clone()) {
                        Ok(_) => {
                            self.preset_status = Some((
                                PresetStatus::Success(format!("Saved: {}.toml", self.preset_input)),
                                Instant::now(),
                            ));
                            // Rebuild explorer to show the new file
                            self.preset_explorer =
                                Self::build_preset_explorer(self.preset_dir.clone());
                        }
                        Err(e) => {
                            self.preset_status =
                                Some((PresetStatus::Error(format!("{}", e)), Instant::now()));
                        }
                    }
                    self.is_preset_input_mode = false;
                }
                self.preset_input.clear();
                self.is_preset_input_mode = false;
            }
            KeyCode::Esc => {
                self.preset_input.clear();
                self.is_preset_input_mode = false;
            }
            _ => {}
        }
    }
    fn handle_key_event(&mut self, key_event: KeyEvent) -> bool {
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
                    return true;
                }
                if self.page == ActivePage::Preset {
                    self.is_preset_input_mode = true;
                    return true;
                }
            }
            Action::Delete => {
                if self.page == ActivePage::Preset {
                    let path = self.preset_explorer.current().path.clone();
                    match PresetManager::delete(&path, self.preset_dir.clone()) {
                        Ok(_) => {
                            self.preset_status = Some((
                                PresetStatus::Success(String::from("Deleted")),
                                Instant::now(),
                            ));
                            // Rebuild explorer to show the new file
                            self.preset_explorer =
                                Self::build_preset_explorer(self.preset_dir.clone());
                        }
                        Err(e) => {
                            self.preset_status =
                                Some((PresetStatus::Error(e.to_string()), Instant::now()));
                        }
                    }
                    return true;
                }
            }
            Action::Select => {
                if self.is_file_explorer_visible && !self.file_explorer.current().is_dir {
                    self.is_image_selected = true;
                    self.is_file_explorer_visible = false;
                    return true;
                }
                if self.page == ActivePage::Preset && !self.preset_explorer.current().is_dir {
                    let path = self.preset_explorer.current().path.clone();
                    match PresetManager::load(&path) {
                        Ok(data) => {
                            PresetManager::apply_to_app_state(
                                &data,
                                &mut self.sliders,
                                &mut self.wheels,
                                &mut self.effects,
                            );
                            self.is_re_render = true;
                            self.selected_effect_index = 0;
                            let name = path
                                .file_stem()
                                .map(|s| s.to_string_lossy().to_string())
                                .unwrap_or_default();
                            self.preset_status = Some((
                                PresetStatus::Success(format!("Loaded: {}", name)),
                                Instant::now(),
                            ));
                        }
                        Err(e) => {
                            self.preset_status = Some((
                                PresetStatus::Error(format!("Error: {}", e)),
                                Instant::now(),
                            ));
                        }
                    }
                }
            }
            Action::SwitchToSliders => {
                self.page = ActivePage::Sliders;
            }
            Action::SwitchToWheels => {
                self.page = ActivePage::Wheels;
            }
            Action::SwitchToScopes => {
                self.page = ActivePage::Scopes;
            }
            Action::SwitchToPipeline => {
                self.page = ActivePage::Pipeline;
            }
            Action::SwitchToPreset => {
                self.page = ActivePage::Preset;
            }
            Action::ToggleLayout => {
                self.layout = match self.layout {
                    AppLayout::Horizontal => AppLayout::Vertical,
                    AppLayout::Vertical => AppLayout::Horizontal,
                };
            }
            Action::NextTool => match self.page {
                ActivePage::Sliders => {
                    self.selected_slider_index =
                        (self.selected_slider_index + 1) % self.sliders.len();
                }
                ActivePage::Wheels => {
                    let current_wheel = &mut self.wheels[self.selected_wheel_index];
                    if current_wheel.focused_part == crate::ui::wheel::SelectedPart::LumSlider {
                        current_wheel.focused_part = crate::ui::wheel::SelectedPart::Wheel;
                        self.selected_wheel_index =
                            (self.selected_wheel_index + 1) % self.wheels.len();
                    } else {
                        current_wheel.focused_part = crate::ui::wheel::SelectedPart::LumSlider;
                    }
                }
                ActivePage::Scopes => {}
                ActivePage::Pipeline => {
                    self.selected_effect_index =
                        (self.selected_effect_index + 1) % self.effects.len();
                }
                ActivePage::Preset => {}
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
                    ActivePage::Scopes => {}
                    ActivePage::Pipeline => {
                        let p = &mut self.effects;
                        let len = p.len();
                        let i = self.selected_effect_index;
                        let direction = if self.layout == AppLayout::Horizontal {
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
                    ActivePage::Preset => {}
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
                    if w.focused_part == crate::ui::wheel::SelectedPart::Wheel {
                        w.x = 0.0;
                        w.y = 0.0;
                    } else {
                        w.lum.state.set_value(w.lum.default_value);
                    }
                    self.is_re_render = true;
                }
                ActivePage::Scopes => {}
                ActivePage::Pipeline => {
                    self.effects = ColorEffects::default();
                    self.selected_effect_index = 0;
                    self.is_re_render = true;
                }
                ActivePage::Preset => {}
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
            Action::ToggleHelp => self.is_help_view = !self.is_help_view,
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
            Action::Escape => {
                self.is_help_view = false;
                self.is_file_explorer_visible = false;
            }
            _ => return false,
        }
        false
    }
}
