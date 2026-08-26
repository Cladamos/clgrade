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

impl ColorGrade {
    pub fn apply(&self, source: &RgbaImage, working: &mut RgbaImage) {
        let exp_mult = 2.0_f32.powf(self.exposure);
        let cont_factor = (259.0 * (self.contrast + 255.0)) / (255.0 * (259.0 - self.contrast));

        let radians = self.hue_degrees.to_radians();
        let cos_a = radians.cos();
        let sin_a = radians.sin();
        let hue_mat = [
            0.213 + 0.787 * cos_a - 0.213 * sin_a,
            0.715 - 0.715 * cos_a - 0.715 * sin_a,
            0.072 - 0.072 * cos_a + 0.928 * sin_a,
            0.213 - 0.213 * cos_a + 0.143 * sin_a,
            0.715 + 0.285 * cos_a + 0.140 * sin_a,
            0.072 - 0.072 * cos_a - 0.283 * sin_a,
            0.213 - 0.213 * cos_a - 0.787 * sin_a,
            0.715 - 0.715 * cos_a + 0.715 * sin_a,
            0.072 + 0.928 * cos_a + 0.072 * sin_a,
        ];

        // Multiplier for the lift, gamma, gain wheels
        let intensity = 50.0;
        // Pre calculate Lift (Shadows) Color + Brightness
        let lift_r = (self.lift_y + self.lift_x) * intensity + self.lift_lum;
        let lift_g = (self.lift_y - self.lift_x) * intensity + self.lift_lum;
        let lift_b = (-self.lift_y + self.lift_x) * intensity + self.lift_lum;

        // Pre calculate Gamma (Midtones) Color + Brightness
        let gamma_r = (self.gamma_y + self.gamma_x) * intensity + self.gamma_lum;
        let gamma_g = (self.gamma_y - self.gamma_x) * intensity + self.gamma_lum;
        let gamma_b = (-self.gamma_y + self.gamma_x) * intensity + self.gamma_lum;

        // Pre calculate Gain (Highlights) Color + Brightness
        let gain_r = (self.gain_y + self.gain_x) * intensity + self.gain_lum;
        let gain_g = (self.gain_y - self.gain_x) * intensity + self.gain_lum;
        let gain_b = (-self.gain_y + self.gain_x) * intensity + self.gain_lum;

        working
            .par_pixels_mut()
            .zip(source.par_pixels())
            .for_each(|(w_px, s_px)| {
                let mut r = s_px[0] as f32;
                let mut g = s_px[1] as f32;
                let mut b = s_px[2] as f32;

                // White Balance (Temperature & Tint)
                r += self.temperature + self.tint;
                g -= self.tint;
                b -= self.temperature - self.tint;

                // Exposure
                r *= exp_mult;
                g *= exp_mult;
                b *= exp_mult;

                // Contrast
                r = cont_factor * (r - 128.0) + 128.0;
                g = cont_factor * (g - 128.0) + 128.0;
                b = cont_factor * (b - 128.0) + 128.0;

                // Lift, gamma, gain
                let mask_lum = ((0.2126 * r + 0.7152 * g + 0.0722 * b) / 255.0).clamp(0.0, 1.0);

                let shadow_mask = (1.0 - (mask_lum * 2.0)).clamp(0.0, 1.0);
                let highlight_mask = ((mask_lum - 0.5) * 2.0).clamp(0.0, 1.0);
                let midtone_mask = (1.0 - shadow_mask - highlight_mask).clamp(0.0, 1.0);

                r += (lift_r * shadow_mask) + (gamma_r * midtone_mask) + (gain_r * highlight_mask);
                g += (lift_g * shadow_mask) + (gamma_g * midtone_mask) + (gain_g * highlight_mask);
                b += (lift_b * shadow_mask) + (gamma_b * midtone_mask) + (gain_b * highlight_mask);

                // Hue Rotation
                if self.hue_degrees != 0.0 {
                    let hr = r * hue_mat[0] + g * hue_mat[1] + b * hue_mat[2];
                    let hg = r * hue_mat[3] + g * hue_mat[4] + b * hue_mat[5];
                    let hb = r * hue_mat[6] + g * hue_mat[7] + b * hue_mat[8];
                    r = hr;
                    g = hg;
                    b = hb;
                }

                // Saturation
                if self.saturation != 1.0 {
                    // Rec. 709 Luminance weights
                    let lum = 0.2126 * r + 0.7152 * g + 0.0722 * b;
                    r = lum + (r - lum) * self.saturation;
                    g = lum + (g - lum) * self.saturation;
                    b = lum + (b - lum) * self.saturation;
                }

                w_px[0] = r.clamp(0.0, 255.0) as u8;
                w_px[1] = g.clamp(0.0, 255.0) as u8;
                w_px[2] = b.clamp(0.0, 255.0) as u8;
                w_px[3] = s_px[3]; // alpha
            });
    }
}

pub struct ImageHandler {
    pub protocol: Option<Protocol>,
    pub image_path: Option<PathBuf>,
    pub loading: bool,
    pub grade: ColorGrade,
    pub target_size: Size,
    pub is_proxy_enabled: bool,

    path: PathBuf,
    resolution: (u32, u32),
    grade_tx: Option<mpsc::Sender<ColorGrade>>,
    protocol_rx: Option<mpsc::Receiver<Protocol>>,
    picker: Picker,
}

impl ImageHandler {
    pub fn new() -> Self {
        let picker = Picker::from_query_stdio().expect("Terminal must support graphics");
        ImageHandler {
            protocol: None,
            image_path: None,
            loading: false,
            grade: ColorGrade::default(),
            target_size: Size::new(26, 12),
            is_proxy_enabled: true,

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
            while let Ok(protocol) = rx.try_recv() {
                latest = Some(protocol);
            }
            if let Some(protocol) = latest {
                self.protocol = Some(protocol);
                self.loading = false;
            }
        }
    }

    pub fn load_from_path(&mut self, path: PathBuf) {
        self.grade_tx = None;

        self.image_path = Some(path.clone());
        self.loading = true;
        self.grade = ColorGrade::default();
        self.path = path.clone();

        let (grade_tx, grade_rx) = mpsc::channel::<ColorGrade>();
        let (protocol_tx, protocol_rx) = mpsc::channel::<Protocol>();

        self.grade_tx = Some(grade_tx);
        self.protocol_rx = Some(protocol_rx);

        let picker = self.picker.clone();
        let resolution = self.resolution;
        let target_size = self.target_size;
        let is_proxy_enabled = self.is_proxy_enabled;

        thread::spawn(move || {
            let dyn_img = image::ImageReader::open(path)
                .expect("Failed to open image")
                .decode()
                .expect("Failed to decode image");
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

            let initial = picker
                .new_protocol(
                    DynamicImage::ImageRgba8(source_high.clone()),
                    target_size,
                    Resize::Scale(Some(Nearest)),
                )
                .unwrap();
            if protocol_tx.send(initial).is_err() {
                return;
            }

            let mut last_grade = ColorGrade::default();
            let mut is_dragging = false;
            let timeout = if is_proxy_enabled {
                Duration::from_millis(200)
            } else {
                Duration::from_millis(16)
            };

            let frame_throttle = Duration::from_millis(16);
            let last_render_time = Instant::now();
            loop {
                let mut grade = if is_dragging {
                    match grade_rx.recv_timeout(timeout) {
                        Ok(g) => g,
                        Err(mpsc::RecvTimeoutError::Timeout) => {
                            is_dragging = false;
                            let mut working_high = source_high.clone();
                            last_grade.apply(&source_high, &mut working_high);
                            let protocol = picker
                                .new_protocol(
                                    DynamicImage::ImageRgba8(working_high),
                                    target_size,
                                    Resize::Scale(Some(Nearest)),
                                )
                                .unwrap();
                            if protocol_tx.send(protocol).is_err() {
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
                    grade = newer;
                }

                last_grade = grade;
                is_dragging = true;

                if last_render_time.elapsed() < frame_throttle {
                    continue;
                }

                if let (Some(sp), Some(wp)) = (source_proxy.as_ref(), working_proxy.as_mut()) {
                    grade.apply(sp, wp);
                    let protocol = picker
                        .new_protocol(
                            DynamicImage::ImageRgba8(wp.clone()),
                            target_size,
                            Resize::Scale(Some(Nearest)),
                        )
                        .unwrap();

                    if protocol_tx.send(protocol).is_err() {
                        break;
                    }
                }
            }
        });
    }

    pub fn save_to_path(&self, mut export_path: PathBuf) {
        let path = self.path.clone();
        let file_name = path.file_name().unwrap();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
        export_path.push(file_name);
        export_path.set_extension(format!("output.{}", ext));

        let grade = self.grade.clone();
        thread::spawn(move || {
            let dyn_img = image::ImageReader::open(path)
                .expect("Failed to open image")
                .decode()
                .expect("Failed to decode image");
            let mut export_image = dyn_img.to_rgba8();
            grade.apply(&dyn_img.to_rgba8(), &mut export_image);
            match DynamicImage::ImageRgba8(export_image).save(&export_path) {
                Ok(_) => {}
                Err(e) => panic!("Failed to save export: {e}"),
            }
        });
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

    pub fn apply_effects(&mut self, grade: ColorGrade) {
        self.grade = grade;
        if let Some(ref tx) = self.grade_tx {
            let _ = tx.send(grade);
        }
    }
}
