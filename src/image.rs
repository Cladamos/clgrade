use std::path::PathBuf;
use std::sync::mpsc;
use std::thread;
use std::time::Duration;

use image::{DynamicImage, RgbaImage};
use ratatui::layout::Size;
use ratatui_image::{FilterType::Nearest, Resize, picker::Picker, protocol::Protocol};
use rayon::prelude::*;

const PREVIEW_MAX_DIM: u32 = 840;
const PROXY_MAX_DIM: u32 = 420;
const TARGET_SIZE: Size = Size::new(60, 28);

#[derive(Clone, Copy, Debug, PartialEq)]
pub struct ColorGrade {
    pub hue_degrees: f32,
}

impl ColorGrade {
    pub fn apply(&self, source: &RgbaImage, working: &mut RgbaImage) {
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

        working
            .par_pixels_mut()
            .zip(source.par_pixels())
            .for_each(|(w_px, s_px)| {
                let r = s_px[0] as f32;
                let g = s_px[1] as f32;
                let b = s_px[2] as f32;

                w_px[0] =
                    (r * hue_mat[0] + g * hue_mat[1] + b * hue_mat[2]).clamp(0.0, 255.0) as u8;
                w_px[1] =
                    (r * hue_mat[3] + g * hue_mat[4] + b * hue_mat[5]).clamp(0.0, 255.0) as u8;
                w_px[2] =
                    (r * hue_mat[6] + g * hue_mat[7] + b * hue_mat[8]).clamp(0.0, 255.0) as u8;
                w_px[3] = s_px[3];
            });
    }
}

pub struct ImageHandler {
    pub protocol: Option<Protocol>,
    pub image_path: Option<PathBuf>,
    pub loading: bool,
    pub grade: ColorGrade,

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
            grade: ColorGrade { hue_degrees: 0.0 },
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
        self.grade = ColorGrade { hue_degrees: 0.0 };

        let (grade_tx, grade_rx) = mpsc::channel::<ColorGrade>();
        let (protocol_tx, protocol_rx) = mpsc::channel::<Protocol>();

        self.grade_tx = Some(grade_tx);
        self.protocol_rx = Some(protocol_rx);

        let picker = self.picker.clone();

        thread::spawn(move || {
            let Ok(dyn_img) = image::ImageReader::open(path).and_then(|r| {
                r.decode()
                    .map_err(|e| std::io::Error::new(std::io::ErrorKind::Other, e))
            }) else {
                return;
            };

            let source_high = dyn_img
                .thumbnail(PREVIEW_MAX_DIM, PREVIEW_MAX_DIM)
                .to_rgba8();

            let source_proxy = dyn_img.thumbnail(PROXY_MAX_DIM, PROXY_MAX_DIM).to_rgba8();
            let mut working_proxy = source_proxy.clone();

            let initial = picker
                .new_protocol(
                    DynamicImage::ImageRgba8(source_high.clone()),
                    TARGET_SIZE,
                    Resize::Scale(Some(Nearest)),
                )
                .unwrap();
            if protocol_tx.send(initial).is_err() {
                return;
            }

            let mut last_grade = ColorGrade { hue_degrees: 0.0 };
            let mut is_dragging = false;
            let timeout = Duration::from_millis(200);

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
                                    TARGET_SIZE,
                                    Resize::Fit(Some(Nearest)),
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

                grade.apply(&source_proxy, &mut working_proxy);
                let protocol = picker
                    .new_protocol(
                        DynamicImage::ImageRgba8(working_proxy.clone()),
                        TARGET_SIZE,
                        Resize::Scale(Some(Nearest)),
                    )
                    .unwrap();

                if protocol_tx.send(protocol).is_err() {
                    break;
                }
            }
        });
    }

    pub fn apply_effects(&mut self, grade: ColorGrade) {
        self.grade = grade;
        if let Some(ref tx) = self.grade_tx {
            let _ = tx.send(grade);
        }
    }

    pub fn has_source(&self) -> bool {
        self.protocol.is_some() || self.loading
    }
}
