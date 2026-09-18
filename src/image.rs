use std::{
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use image::{DynamicImage, RgbaImage};
use ratatui::layout::Size;
use ratatui_image::{FilterType::Nearest, Resize, picker::Picker, protocol::Protocol};
use rayon::prelude::*;

use crate::ui::{color_mixer::ColorMixerPart, pipeline::ColorEffects};

//TODO: Definitely need to refactor color effects and handling them maybe move to another file
#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorGrade {
    pub temperature: f32, // -100.0 to 100.0
    pub tint: f32,        // -100.0 to 100.0
    pub exposure: f32,    // -3.0 to 3.0
    pub contrast: f32,    // -100.0 to 100.0
    pub saturation: f32,  // 0.0 to 2.0
    pub hue_degrees: f32, // -180.0 to 180.0

    pub lift_x: f32,    // -1.0 to 1.0
    pub lift_y: f32,    // -1.0 to 1.0
    pub lift_lum: f32,  // -100.0 to 100.0
    pub gamma_x: f32,   // -1.0 to 1.0
    pub gamma_y: f32,   // -1.0 to 1.0
    pub gamma_lum: f32, // -100.0 to 100.0
    pub gain_x: f32,    // -1.0 to 1.0
    pub gain_y: f32,    // -1.0 to 1.0
    pub gain_lum: f32,  // -100.0 to 100.0
}

impl Default for ColorGrade {
    fn default() -> Self {
        ColorGrade {
            temperature: 0.0,
            tint: 0.0,
            exposure: 0.0,
            contrast: 0.0,
            saturation: 1.0,
            hue_degrees: 0.0,

            lift_x: 0.0,
            lift_y: 0.0,
            lift_lum: 0.0,
            gamma_x: 0.0,
            gamma_y: 0.0,
            gamma_lum: 0.0,
            gain_x: 0.0,
            gain_y: 0.0,
            gain_lum: 0.0,
        }
    }
}

pub struct PrecomputedParams<'a> {
    pub temperature: f32,
    pub tint: f32,
    pub exp_mult: f32,
    pub cont_factor: f32,
    pub hue_degrees: f32,
    pub hue_mat: [f32; 9],
    pub saturation: f32,
    pub lift_r: f32,
    pub lift_g: f32,
    pub lift_b: f32,
    pub gamma_r: f32,
    pub gamma_g: f32,
    pub gamma_b: f32,
    pub gain_r: f32,
    pub gain_g: f32,
    pub gain_b: f32,
    pub hsl_adjustments: &'a [ColorMixerPart],
    pub color_centers: [f32; 8],
    pub blend_degrees: [f32; 8],
}

impl ColorEffects {
    pub fn apply(&self, r: &mut f32, g: &mut f32, b: &mut f32, p: &PrecomputedParams) {
        match self {
            ColorEffects::WhiteBalance => {
                *r += p.temperature + p.tint;
                *g -= p.tint;
                *b -= p.temperature - p.tint;
            }
            ColorEffects::Exposure => {
                *r *= p.exp_mult;
                *g *= p.exp_mult;
                *b *= p.exp_mult;
            }
            ColorEffects::Contrast => {
                *r = p.cont_factor * (*r - 128.0) + 128.0;
                *g = p.cont_factor * (*g - 128.0) + 128.0;
                *b = p.cont_factor * (*b - 128.0) + 128.0;
            }
            ColorEffects::LiftGammaGain => {
                let mask_lum = ((0.2126 * *r + 0.7152 * *g + 0.0722 * *b) / 255.0).clamp(0.0, 1.0);
                let shadow_mask = (1.0 - (mask_lum * 2.0)).clamp(0.0, 1.0);
                let highlight_mask = ((mask_lum - 0.5) * 2.0).clamp(0.0, 1.0);
                let midtone_mask = (1.0 - shadow_mask - highlight_mask).clamp(0.0, 1.0);

                *r += (p.lift_r * shadow_mask)
                    + (p.gamma_r * midtone_mask)
                    + (p.gain_r * highlight_mask);
                *g += (p.lift_g * shadow_mask)
                    + (p.gamma_g * midtone_mask)
                    + (p.gain_g * highlight_mask);
                *b += (p.lift_b * shadow_mask)
                    + (p.gamma_b * midtone_mask)
                    + (p.gain_b * highlight_mask);
            }
            ColorEffects::Hue => {
                if p.hue_degrees != 0.0 {
                    let new_r = *r * p.hue_mat[0] + *g * p.hue_mat[1] + *b * p.hue_mat[2];
                    let new_g = *r * p.hue_mat[3] + *g * p.hue_mat[4] + *b * p.hue_mat[5];
                    let new_b = *r * p.hue_mat[6] + *g * p.hue_mat[7] + *b * p.hue_mat[8];
                    *r = new_r;
                    *g = new_g;
                    *b = new_b;
                }
            }
            ColorEffects::Saturation => {
                if p.saturation != 1.0 {
                    // Rec. 709 Luminance weights
                    let lum = 0.2126 * *r + 0.7152 * *g + 0.0722 * *b;
                    *r = lum + (*r - lum) * p.saturation;
                    *g = lum + (*g - lum) * p.saturation;
                    *b = lum + (*b - lum) * p.saturation;
                }
            }
            //TODO: solve that unnecessary conversions
            ColorEffects::ColorMixer => {
                let (mut h, mut s, mut l) = rgb_to_hsl(
                    r.clamp(0.0, 255.0),
                    g.clamp(0.0, 255.0),
                    b.clamp(0.0, 255.0),
                );
                // Only attempt to shift color if the pixel actually has color (skip pure black/white)
                if s > 0.01 {
                    let mut hue_shift = 0.0;
                    let mut sat_mult = 1.0;
                    let mut luma_offset = 0.0;

                    for (i, mixer_part) in p.hsl_adjustments.iter().enumerate() {
                        // Find shortest distance on a circle (e.g., Red at 0 is close to Magenta at 350)
                        let mut dist = (h - p.color_centers[i]).abs();
                        if dist > 180.0 {
                            dist = 360.0 - dist;
                        }

                        // If the pixel is close enough to the target color, calculate its weight (0.0 to 1.0)
                        if dist < p.blend_degrees[i] {
                            let weight = 1.0 - (dist / p.blend_degrees[i]);
                            hue_shift += mixer_part.sliders[0].state.value() as f32 * weight;
                            let s_target = mixer_part.sliders[1].state.value() as f32;
                            sat_mult *= 1.0 + ((s_target - 1.0) * weight);
                            // Divided by 100 because of I am getting slider data (-100, 100) and I need (-1, 1) for hsl_to_rgb
                            luma_offset +=
                                (mixer_part.sliders[2].state.value() / 100.0) as f32 * weight;
                        }
                    }

                    h += hue_shift;
                    h %= 360.0;
                    if h < 0.0 {
                        h += 360.0;
                    }

                    s = (s * sat_mult).clamp(0.0, 1.0);
                    l = (l + luma_offset).clamp(0.0, 1.0);
                }
                let (new_r, new_g, new_b) = hsl_to_rgb(h, s, l);
                *r = new_r;
                *g = new_g;
                *b = new_b;
            }
        }
    }
}

// Converts RGB (0.0 - 255.0) to HSL (H: 0-360, S: 0.0-1.0, L: 0.0-1.0)
#[inline(always)]
fn rgb_to_hsl(r: f32, g: f32, b: f32) -> (f32, f32, f32) {
    let r_norm = r / 255.0;
    let g_norm = g / 255.0;
    let b_norm = b / 255.0;

    let max = r_norm.max(g_norm).max(b_norm);
    let min = r_norm.min(g_norm).min(b_norm);
    let delta = max - min;

    let l = (max + min) / 2.0;
    let mut h = 0.0;
    let mut s = 0.0;

    if delta != 0.0 {
        s = if l < 0.5 {
            delta / (max + min)
        } else {
            delta / (2.0 - max - min)
        };

        if max == r_norm {
            h = (g_norm - b_norm) / delta + (if g_norm < b_norm { 6.0 } else { 0.0 });
        } else if max == g_norm {
            h = (b_norm - r_norm) / delta + 2.0;
        } else {
            h = (r_norm - g_norm) / delta + 4.0;
        }
        h *= 60.0;
    }

    (h, s, l)
}

// Converts HSL (H: 360 , S: 0.0-1.0, L: 0.0-1.0) back to RGB (0.0 - 255.0)
#[inline(always)]
fn hsl_to_rgb(h: f32, s: f32, l: f32) -> (f32, f32, f32) {
    if s == 0.0 {
        let v = l * 255.0;
        return (v, v, v);
    }

    let q = if l < 0.5 {
        l * (1.0 + s)
    } else {
        l + s - l * s
    };
    let p = 2.0 * l - q;

    let hk = h / 360.0;
    let mut rgb = [hk + 1.0 / 3.0, hk, hk - 1.0 / 3.0];

    for c in &mut rgb {
        if *c < 0.0 {
            *c += 1.0;
        }
        if *c > 1.0 {
            *c -= 1.0;
        }

        *c = if *c < 1.0 / 6.0 {
            p + (q - p) * 6.0 * (*c)
        } else if *c < 1.0 / 2.0 {
            q
        } else if *c < 2.0 / 3.0 {
            p + (q - p) * (2.0 / 3.0 - *c) * 6.0
        } else {
            p
        };
    }

    (rgb[0] * 255.0, rgb[1] * 255.0, rgb[2] * 255.0)
}
impl ColorGrade {
    pub fn apply(
        &self,
        source: &RgbaImage,
        working: &mut RgbaImage,
        pipeline: &[ColorEffects],
        color_mixer: &[ColorMixerPart],
    ) {
        let radians = self.hue_degrees.to_radians();
        let cos_a = radians.cos();
        let sin_a = radians.sin();

        // Multiplier for the lift, gamma, gain wheels
        let intensity = 50.0;

        let params = PrecomputedParams {
            temperature: self.temperature,
            tint: self.tint,
            exp_mult: 2.0_f32.powf(self.exposure),
            cont_factor: (259.0 * (self.contrast + 255.0)) / (255.0 * (259.0 - self.contrast)),
            hue_degrees: self.hue_degrees,
            hue_mat: [
                0.213 + 0.787 * cos_a - 0.213 * sin_a,
                0.715 - 0.715 * cos_a - 0.715 * sin_a,
                0.072 - 0.072 * cos_a + 0.928 * sin_a,
                0.213 - 0.213 * cos_a + 0.143 * sin_a,
                0.715 + 0.285 * cos_a + 0.140 * sin_a,
                0.072 - 0.072 * cos_a - 0.283 * sin_a,
                0.213 - 0.213 * cos_a - 0.787 * sin_a,
                0.715 - 0.715 * cos_a + 0.715 * sin_a,
                0.072 + 0.928 * cos_a + 0.072 * sin_a,
            ],
            saturation: self.saturation,
            // Lift (Shadows)
            lift_r: (self.lift_y + self.lift_x) * intensity + self.lift_lum,
            lift_g: (self.lift_y - self.lift_x) * intensity + self.lift_lum,
            lift_b: (-self.lift_y + self.lift_x) * intensity + self.lift_lum,
            // Gamma (Midtones)
            gamma_r: (self.gamma_y + self.gamma_x) * intensity + self.gamma_lum,
            gamma_g: (self.gamma_y - self.gamma_x) * intensity + self.gamma_lum,
            gamma_b: (-self.gamma_y + self.gamma_x) * intensity + self.gamma_lum,
            // Gain (Highlights)
            gain_r: (self.gain_y + self.gain_x) * intensity + self.gain_lum,
            gain_g: (self.gain_y - self.gain_x) * intensity + self.gain_lum,
            gain_b: (-self.gain_y + self.gain_x) * intensity + self.gain_lum,

            // The exact angles for the 8 color targets on a 360-degree wheel:
            // 0: Red, 1: Orange, 2: Yellow, 3: Green, 4: Aqua, 5: Blue, 6: Purple, 7: Magenta
            color_centers: [0.0, 30.0, 60.0, 120.0, 180.0, 240.0, 270.0, 300.0],
            blend_degrees: [30.0, 20.0, 30.0, 60.0, 60.0, 45.0, 20.0, 30.0],

            hsl_adjustments: color_mixer,
        };

        working
            .par_pixels_mut()
            .zip(source.par_pixels())
            .for_each(|(w_px, s_px)| {
                let mut r = s_px[0] as f32;
                let mut g = s_px[1] as f32;
                let mut b = s_px[2] as f32;

                for effect in pipeline {
                    effect.apply(&mut r, &mut g, &mut b, &params);
                }

                w_px[0] = r.clamp(0.0, 255.0) as u8;
                w_px[1] = g.clamp(0.0, 255.0) as u8;
                w_px[2] = b.clamp(0.0, 255.0) as u8;
                w_px[3] = s_px[3]; // alpha
            });
    }
}
pub struct ScopeData {
    pub vectorscope_points: Vec<(f32, f32)>, // Vectorscope: (x, y)
    pub lum_histogram: [u32; 256],           // Waveform: histogram data
}
impl ScopeData {
    fn new() -> Self {
        ScopeData {
            vectorscope_points: Vec::new(),
            lum_histogram: [0; 256],
        }
    }
}

type ProtocolMessage = Result<(Protocol, Option<ScopeData>), String>;

pub struct ImageHandler {
    pub protocol: Option<Protocol>,
    pub image_path: Option<PathBuf>,
    pub loading: bool,
    pub load_error: Option<String>,
    pub grade: ColorGrade,
    pub pipeline: Vec<ColorEffects>,
    pub color_mixer: Vec<ColorMixerPart>,
    pub target_size: Size,
    pub is_proxy_enabled: bool,
    pub scope_data: ScopeData,

    path: PathBuf,
    resolution: (u32, u32),
    //TODO: Made struct for that also think about naming grade to effect
    grade_tx: Option<mpsc::Sender<(ColorGrade, Vec<ColorEffects>, Vec<ColorMixerPart>)>>,
    protocol_rx: Option<mpsc::Receiver<ProtocolMessage>>,
    picker: Picker,
}

impl ImageHandler {
    pub fn new() -> Self {
        let picker = match Picker::from_query_stdio() {
            Ok(p) => p,
            Err(e) => {
                ratatui::restore();
                eprintln!("Error: Terminal graphics protocol query failed: {e}");
                eprintln!(
                    "\nPlease ensure you are using a terminal emulator with graphics support (e.g. Ghostty, Kitty, WezTerm, iTerm2, or Konsole)."
                );
                std::process::exit(1);
            }
        };
        ImageHandler {
            protocol: None,
            image_path: None,
            loading: false,
            load_error: None,
            grade: ColorGrade::default(),
            pipeline: ColorEffects::default_pipeline(),
            color_mixer: ColorMixerPart::default_parts(),
            target_size: Size::new(17, 8),
            is_proxy_enabled: false,
            scope_data: ScopeData::new(),
            path: PathBuf::new(),
            resolution: (360, 360),
            grade_tx: None,
            protocol_rx: None,
            picker,
        }
    }

    pub fn poll(&mut self) {
        if let Some(ref rx) = self.protocol_rx {
            let mut latest = None;
            while let Ok(data) = rx.try_recv() {
                latest = Some(data);
            }
            match latest {
                Some(Ok((protocol, scope_data))) => {
                    self.protocol = Some(protocol);
                    if let Some(scope_data) = scope_data {
                        self.scope_data = scope_data;
                    }
                    self.loading = false;
                }
                Some(Err(e)) => {
                    self.load_error = Some(e);
                    self.loading = false;
                    self.protocol_rx = None;
                }
                None => {}
            }
        }
    }

    pub fn load_from_path(&mut self, path: PathBuf) {
        self.grade_tx = None;
        self.load_error = None;

        self.image_path = Some(path.clone());
        self.loading = true;
        self.path = path.clone();

        let (grade_tx, grade_rx) =
            mpsc::channel::<(ColorGrade, Vec<ColorEffects>, Vec<ColorMixerPart>)>();
        let (protocol_tx, protocol_rx) = mpsc::channel::<ProtocolMessage>();

        self.grade_tx = Some(grade_tx);
        self.protocol_rx = Some(protocol_rx);

        let grade = self.grade;
        let pipeline = self.pipeline.clone();
        let color_mixer = self.color_mixer.clone();

        let picker = self.picker.clone();
        let resolution = self.resolution;
        let target_size = self.target_size;
        let is_proxy_enabled = self.is_proxy_enabled;

        thread::spawn(move || {
            let dyn_img = match image::ImageReader::open(&path)
                .map_err(|e| format!("Failed to open: {e}"))
                .and_then(|r| r.decode().map_err(|e| format!("Failed to decode: {e}")))
            {
                Ok(img) => img,
                Err(e) => {
                    let _ = protocol_tx.send(Err(e));
                    return;
                }
            };
            let source_high = dyn_img.thumbnail(resolution.0, resolution.1).to_rgba8();
            let source_proxy = if is_proxy_enabled {
                Some(
                    dyn_img
                        .thumbnail(resolution.0 / 2, resolution.1 / 2)
                        .to_rgba8(),
                )
            } else {
                None
            };
            let mut working_proxy = source_proxy.clone();
            let mut working_high = source_high.clone();

            grade.apply(&source_high, &mut working_high, &pipeline, &color_mixer);
            let initial_scope =
                Self::calculate_scopes(&working_high, working_high.width(), working_high.height());
            let initial = picker
                .new_protocol(
                    DynamicImage::ImageRgba8(working_high),
                    target_size,
                    Resize::Scale(Some(Nearest)),
                )
                .unwrap();
            if protocol_tx
                .send(Ok((initial, Some(initial_scope))))
                .is_err()
            {
                return;
            }

            let mut last_grade = ColorGrade::default();
            let mut last_pipeline = ColorEffects::default_pipeline();
            let mut last_color_mixer = ColorMixerPart::default_parts();

            let mut is_dragging = false;
            let timeout = if is_proxy_enabled {
                Duration::from_millis(200)
            } else {
                Duration::from_millis(16)
            };

            let frame_throttle = Duration::from_millis(16);
            let last_render_time = Instant::now();
            loop {
                let (mut grade, mut pipeline, mut color_mixer) = if is_dragging {
                    match grade_rx.recv_timeout(timeout) {
                        Ok((g, p, c)) => (g, p, c),
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            is_dragging = false;
                            working_high = source_high.clone();
                            last_grade.apply(
                                &source_high,
                                &mut working_high,
                                &last_pipeline,
                                &last_color_mixer,
                            );

                            let scope = Self::calculate_scopes(
                                &working_high,
                                working_high.width(),
                                working_high.height(),
                            );
                            let protocol = picker
                                .new_protocol(
                                    DynamicImage::ImageRgba8(working_high),
                                    target_size,
                                    Resize::Scale(Some(Nearest)),
                                )
                                .unwrap();

                            if protocol_tx.send(Ok((protocol, Some(scope)))).is_err() {
                                break;
                            }
                            continue;
                        }
                        Err(mpsc::RecvTimeoutError::Disconnected) => break,
                    }
                } else {
                    match grade_rx.recv() {
                        Ok(g) => g,
                        Err(_) => break,
                    }
                };

                while let Ok(newer) = grade_rx.try_recv() {
                    (grade, pipeline, color_mixer) = newer;
                }

                last_grade = grade;
                last_pipeline = pipeline.clone();
                last_color_mixer = color_mixer.clone();
                is_dragging = true;

                if last_render_time.elapsed() < frame_throttle {
                    continue;
                }

                if let (Some(sp), Some(wp)) = (source_proxy.as_ref(), working_proxy.as_mut()) {
                    grade.apply(sp, wp, &pipeline, &color_mixer);
                    let protocol = picker
                        .new_protocol(
                            DynamicImage::ImageRgba8(wp.clone()),
                            target_size,
                            Resize::Scale(Some(Nearest)),
                        )
                        .unwrap();

                    if protocol_tx.send(Ok((protocol, None))).is_err() {
                        break;
                    }
                }
            }
        });
    }

    pub fn calculate_scopes(image_buffer: &[u8], width: u32, height: u32) -> ScopeData {
        let mut vectorscope_points = Vec::with_capacity(512);
        let mut lum_histogram = [0; 256];

        let total_pixels = width * height;
        let target_samples = 2500;

        let mut pixel_step = (total_pixels / target_samples).max(1);

        // Fix getting pixels from same col if width is divisible by pixel_step
        if pixel_step.is_multiple_of(2) {
            pixel_step += 1;
        }

        const PB_DIVISOR: f32 = 255.0 * 1.8556;
        const PR_DIVISOR: f32 = 255.0 * 1.5748;
        const SCOPE_SCALE: f32 = 1.678;

        // We multiply pixel_step by 4 because each pixel is 4 bytes (R, G, B, A)
        for i in (0..image_buffer.len()).step_by(pixel_step as usize * 4) {
            let r = image_buffer[i] as f32;
            let g = image_buffer[i + 1] as f32;
            let b = image_buffer[i + 2] as f32;

            let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
            let bucket = lum.round() as usize;
            lum_histogram[bucket.clamp(0, 255)] += 1;

            // Scaling to -1.0 to 1.0 grid
            let pb = (b - lum) / PB_DIVISOR * SCOPE_SCALE; // Blue difference
            let pr = (r - lum) / PR_DIVISOR * SCOPE_SCALE; // Red difference

            vectorscope_points.push((pb, pr));
        }

        ScopeData {
            vectorscope_points,
            lum_histogram,
        }
    }

    pub fn save_to_path(&self, mut export_path: PathBuf) -> mpsc::Receiver<Result<String, String>> {
        let path = self.path.clone();
        let (tx, rx) = mpsc::channel();
        if self.protocol.is_none() {
            let _ = tx.send(Err("No protocol available".to_string()));
            return rx;
        }
        if !path.exists() {
            let _ = tx.send(Err("Path does not exist".to_string()));
            return rx;
        }
        if path.file_name().is_none() || path.extension().is_none() {
            let _ = tx.send(Err("Invalid path".to_string()));
            return rx;
        }

        let pipeline = self.pipeline.clone();
        let color_mixer = self.color_mixer.clone();
        let file_name = path.file_name().unwrap();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
        export_path.push(file_name);
        export_path.set_extension(format!("output.{}", ext));

        let grade = self.grade;
        let export_path_str = export_path.to_str().unwrap_or_default().to_string();
        thread::spawn(move || {
            let result = (|| {
                let dyn_img = image::ImageReader::open(path)
                    .map_err(|e| format!("Failed to open: {e}"))?
                    .decode()
                    .map_err(|e| format!("Failed to decode: {e}"))?;
                let mut export_image = dyn_img.to_rgba8();
                grade.apply(
                    &dyn_img.to_rgba8(),
                    &mut export_image,
                    &pipeline,
                    &color_mixer,
                );
                DynamicImage::ImageRgba8(export_image)
                    .save(&export_path)
                    .map_err(|e| format!("Failed to save: {e}"))?;
                Ok(export_path_str)
            })();
            let _ = tx.send(result);
        });
        rx
    }

    pub fn set_resolution(&mut self, resolution: u32, aspect_ratio: (u8, u8)) {
        let (width, height);
        if aspect_ratio.1 > aspect_ratio.0 {
            width = resolution;
            height = (resolution as f32 * aspect_ratio.1 as f32 / aspect_ratio.0 as f32) as u32;
        } else {
            width = (resolution as f32 * aspect_ratio.0 as f32 / aspect_ratio.1 as f32) as u32;
            height = resolution;
        }

        self.resolution = (width, height);

        let font_size = self.picker.font_size();
        self.target_size = Size::new(
            (width / font_size.width as u32) as u16,
            (height / font_size.height as u32) as u16,
        );
    }

    pub fn reload(&mut self) {
        if let Some(path) = self.image_path.clone() {
            self.load_from_path(path);
        }
    }

    pub fn apply_effects(
        &mut self,
        grade: ColorGrade,
        pipeline: Vec<ColorEffects>,
        color_mixer: Vec<ColorMixerPart>,
    ) {
        self.grade = grade;
        self.pipeline = pipeline.clone();
        self.color_mixer = color_mixer.clone();

        if let Some(ref tx) = self.grade_tx {
            let _ = tx.send((grade, pipeline, color_mixer));
        }
    }
}
