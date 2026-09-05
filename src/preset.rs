use serde::{Deserialize, Serialize};
use std::io::{self, Error, ErrorKind};
use std::path::{Path, PathBuf};

use crate::ui::pipeline::ColorEffects;
use crate::ui::slider::SliderData;
use crate::ui::wheel::WheelData;

#[derive(Serialize, Deserialize)]
pub struct PresetData {
    pub sliders: SliderPreset,
    pub wheels: WheelsPreset,
    pub pipeline: PipelinePreset,
}

#[derive(Serialize, Deserialize)]
pub struct SliderPreset {
    pub temperature: f64,
    pub tint: f64,
    pub exposure: f64,
    pub contrast: f64,
    pub saturation: f64,
    pub hue: f64,
}

#[derive(Serialize, Deserialize, Default)]
pub struct WheelPreset {
    pub x: f32,
    pub y: f32,
    pub lum: f64,
}

#[derive(Serialize, Deserialize)]
pub struct WheelsPreset {
    pub lift: WheelPreset,
    pub gamma: WheelPreset,
    pub gain: WheelPreset,
}

#[derive(Serialize, Deserialize)]
pub struct PipelinePreset {
    pub order: Vec<String>,
}

pub struct PresetManager;

impl PresetManager {
    pub fn create_presets_dir() -> PathBuf {
        let dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clgrade")
            .join("presets");
        std::fs::create_dir_all(&dir).ok();
        dir
    }

    pub fn save(name: &str, data: &PresetData, path: PathBuf) -> io::Result<()> {
        let path = path.join(format!("{}.toml", name));
        //TODO: gave overwrite option to user
        if path.exists() {
            return Err(Error::new(
                ErrorKind::AlreadyExists,
                "Preset already exists",
            ));
        }
        let content =
            toml::to_string_pretty(data).map_err(|e| Error::new(io::ErrorKind::Other, e))?;
        std::fs::write(path, content)
    }

    pub fn load(path: &Path) -> io::Result<PresetData> {
        let content = std::fs::read_to_string(path)?;
        toml::from_str(&content).map_err(|e| Error::new(io::ErrorKind::Other, e))
    }

    pub fn delete(path: &Path, preset_dir: PathBuf) -> io::Result<()> {
        if !path.exists() {
            return Err(Error::new(io::ErrorKind::NotFound, "File not found"));
        }
        if path.canonicalize().unwrap().parent() == Some(&preset_dir.canonicalize().unwrap())
            && path.is_file()
        {
            std::fs::remove_file(path)
        } else {
            Err(Error::new(
                io::ErrorKind::PermissionDenied,
                "Cannot delete file outside presets folder",
            ))
        }
    }

    pub fn from_app_state(
        sliders: &[SliderData],
        wheels: &[WheelData],
        effects: &[ColorEffects],
    ) -> PresetData {
        let slider_val = |label: &str| {
            sliders
                .iter()
                .find(|s| s.label == label)
                .map(|s| s.state.value())
                .unwrap()
        };

        let wheel_val = |prefix: &str| {
            wheels
                .iter()
                .find(|w| w.label.starts_with(prefix))
                .map(|w| WheelPreset {
                    x: w.x,
                    y: w.y,
                    lum: w.lum.state.value(),
                })
                .unwrap()
        };

        PresetData {
            sliders: SliderPreset {
                temperature: slider_val("Temp"),
                tint: slider_val("Tint"),
                exposure: slider_val("Exp"),
                contrast: slider_val("Cont"),
                saturation: slider_val("Sat"),
                hue: slider_val("Hue"),
            },
            wheels: WheelsPreset {
                lift: wheel_val("Lift"),
                gamma: wheel_val("Gamma"),
                gain: wheel_val("Gain"),
            },
            pipeline: PipelinePreset {
                order: effects
                    .iter()
                    .map(|e| e.get_short_name().to_string())
                    .collect(),
            },
        }
    }

    pub fn apply_to_app_state(
        data: &PresetData,
        sliders: &mut [SliderData],
        wheels: &mut [WheelData],
        effects: &mut Vec<ColorEffects>,
    ) {
        for slider in sliders.iter_mut() {
            match slider.label {
                "Temp" => slider.state.set_value(data.sliders.temperature),
                "Tint" => slider.state.set_value(data.sliders.tint),
                "Exp" => slider.state.set_value(data.sliders.exposure),
                "Cont" => slider.state.set_value(data.sliders.contrast),
                "Sat" => slider.state.set_value(data.sliders.saturation),
                "Hue" => slider.state.set_value(data.sliders.hue),
                _ => {}
            }
        }

        for wheel in wheels.iter_mut() {
            match wheel.label {
                "Lift (Shadows)" => {
                    wheel.x = data.wheels.lift.x;
                    wheel.y = data.wheels.lift.y;
                    wheel.lum.state.set_value(data.wheels.lift.lum);
                }
                "Gamma (Midtones)" => {
                    wheel.x = data.wheels.gamma.x;
                    wheel.y = data.wheels.gamma.y;
                    wheel.lum.state.set_value(data.wheels.gamma.lum);
                }
                "Gain (Highlights)" => {
                    wheel.x = data.wheels.gain.x;
                    wheel.y = data.wheels.gain.y;
                    wheel.lum.state.set_value(data.wheels.gain.lum);
                }
                _ => {}
            }
        }

        // Pipeline order
        let mut new_effects: Vec<ColorEffects> = Vec::new();
        for name in &data.pipeline.order {
            if let Some(effect) = ColorEffects::from_short_name(name) {
                new_effects.push(effect);
            }
        }
        if !new_effects.is_empty() {
            *effects = new_effects;
        }
    }
}
