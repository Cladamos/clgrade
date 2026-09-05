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

#[derive(Serialize, Deserialize)]
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
    pub fn presets_dir() -> PathBuf {
        let dir = dirs::config_dir()
            .unwrap_or_else(|| PathBuf::from("."))
            .join("clgrade")
            .join("presets");
        std::fs::create_dir_all(&dir).ok();
        dir
    }

    pub fn save(name: &str, data: &PresetData) -> io::Result<()> {
        let path = Self::presets_dir().join(format!("{}.toml", name));
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

    pub fn delete(path: &Path) -> io::Result<()> {
        if !path.exists() {
            return Err(Error::new(io::ErrorKind::NotFound, "File not found"));
        }
        if path.parent() == Some(&Self::presets_dir()) && path.is_file() {
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
        PresetData {
            sliders: SliderPreset {
                temperature: sliders[0].state.value(),
                tint: sliders[1].state.value(),
                exposure: sliders[2].state.value(),
                contrast: sliders[3].state.value(),
                saturation: sliders[4].state.value(),
                hue: sliders[5].state.value(),
            },
            wheels: WheelsPreset {
                lift: WheelPreset {
                    x: wheels[0].x,
                    y: wheels[0].y,
                    lum: wheels[0].lum.state.value(),
                },
                gamma: WheelPreset {
                    x: wheels[1].x,
                    y: wheels[1].y,
                    lum: wheels[1].lum.state.value(),
                },
                gain: WheelPreset {
                    x: wheels[2].x,
                    y: wheels[2].y,
                    lum: wheels[2].lum.state.value(),
                },
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
        sliders[0].state.set_value(data.sliders.temperature);
        sliders[1].state.set_value(data.sliders.tint);
        sliders[2].state.set_value(data.sliders.exposure);
        sliders[3].state.set_value(data.sliders.contrast);
        sliders[4].state.set_value(data.sliders.saturation);
        sliders[5].state.set_value(data.sliders.hue);

        wheels[0].x = data.wheels.lift.x;
        wheels[0].y = data.wheels.lift.y;
        wheels[0].lum.state.set_value(data.wheels.lift.lum);
        wheels[1].x = data.wheels.gamma.x;
        wheels[1].y = data.wheels.gamma.y;
        wheels[1].lum.state.set_value(data.wheels.gamma.lum);
        wheels[2].x = data.wheels.gain.x;
        wheels[2].y = data.wheels.gain.y;
        wheels[2].lum.state.set_value(data.wheels.gain.lum);

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
