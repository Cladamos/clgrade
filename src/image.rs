use std::{
    path::PathBuf,
    sync::mpsc,
    thread,
    time::{Duration, Instant},
};

use image::{DynamicImage, metadata::Orientation};
use ratatui::layout::Size;
use ratatui_image::{FilterType::Nearest, Resize, picker::Picker, protocol::Protocol};

use crate::{
    effect::{CropArea, SliderDatas, WheelDatas, apply_all_effects},
    ui::{color_mixer::ColorMixerPart, pipeline::ColorEffects},
};

//TODO: Definitely need to refactor color effects and handling them maybe move to another file

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
type EffectMessage = (
    SliderDatas,
    WheelDatas,
    Vec<ColorEffects>,
    Vec<ColorMixerPart>,
    CropArea,
);
pub struct ImageHandler {
    pub protocol: Option<Protocol>,
    pub image_path: Option<PathBuf>,
    pub loading: bool,
    pub load_error: Option<String>,
    pub slider_datas: SliderDatas,
    pub wheel_datas: WheelDatas,
    pub pipeline: Vec<ColorEffects>,
    pub color_mixer: Vec<ColorMixerPart>,
    pub crop_area: CropArea,
    pub crop_region: Option<CropArea>,
    pub target_size: Size,
    pub is_proxy_enabled: bool,
    pub scope_data: ScopeData,
    pub resolution: (u32, u32),
    pub source_size: (u32, u32),

    path: PathBuf,
    rotation: Orientation,
    effect_tx: Option<mpsc::Sender<EffectMessage>>,
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
        let initial_resolution = (360, 360);
        ImageHandler {
            protocol: None,
            image_path: None,
            loading: false,
            load_error: None,
            slider_datas: SliderDatas::default(),
            wheel_datas: WheelDatas::default(),
            pipeline: ColorEffects::default_pipeline(),
            color_mixer: ColorMixerPart::default_parts(),
            crop_area: CropArea::default(initial_resolution),
            crop_region: None,
            target_size: Size::new(17, 8),
            is_proxy_enabled: false,
            scope_data: ScopeData::new(),
            path: PathBuf::new(),
            resolution: initial_resolution,
            source_size: (0, 0),
            rotation: Orientation::NoTransforms,
            effect_tx: None,
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

    //TODO: I am doing almost all image operations under this function needs refactoring
    pub fn load_from_path(&mut self, path: PathBuf) {
        self.effect_tx = None;
        self.load_error = None;

        self.image_path = Some(path.clone());
        self.loading = true;
        self.path = path.clone();

        if let Ok(dims) = image::image_dimensions(&path) {
            self.source_size = dims;
        }

        if let Some(ref crop) = self.crop_region {
            self.source_size = (crop.width, crop.height);
        }

        let (effect_tx, effect_rx) = mpsc::channel::<EffectMessage>();
        let (protocol_tx, protocol_rx) = mpsc::channel::<ProtocolMessage>();

        self.effect_tx = Some(effect_tx);
        self.protocol_rx = Some(protocol_rx);

        let sliders = self.slider_datas;
        let wheels = self.wheel_datas;
        let pipeline = self.pipeline.clone();
        let color_mixer = self.color_mixer.clone();

        let picker = self.picker.clone();
        let resolution = self.resolution;
        let source_size = self.source_size;
        let rotation = self.rotation;
        let target_size = self.target_size;
        let is_proxy_enabled = self.is_proxy_enabled;
        let crop_region = self.crop_region;

        thread::spawn(move || {
            let mut dyn_img = match image::ImageReader::open(&path)
                .map_err(|e| format!("Failed to open: {e}"))
                .and_then(|r| r.decode().map_err(|e| format!("Failed to decode: {e}")))
            {
                Ok(img) => img,
                Err(e) => {
                    let _ = protocol_tx.send(Err(e));
                    return;
                }
            };
            let crop_area = CropArea::default(source_size);
            if rotation != Orientation::NoTransforms {
                dyn_img.apply_orientation(rotation);
            }
            if let Some(crop) = crop_region {
                let cropped = dyn_img.crop(crop.x, crop.y, crop.width, crop.height);
                dyn_img = cropped;
            }
            let source_high = dyn_img.thumbnail(resolution.0, resolution.1).to_rgba8();
            // thumbnail() preserves aspect ratio so actual size may differ from resolution
            let actual_high_size = (source_high.width(), source_high.height());
            let source_proxy = if is_proxy_enabled {
                Some(
                    dyn_img
                        .thumbnail(resolution.0 / 2, resolution.1 / 2)
                        .to_rgba8(),
                )
            } else {
                None
            };
            let actual_proxy_size = source_proxy
                .as_ref()
                .map(|p| (p.width(), p.height()))
                .unwrap_or((0, 0));
            let mut working_proxy = source_proxy.clone();
            let mut working_high = source_high.clone();

            let preview_crop = crop_area.scaled(source_size, actual_high_size);
            apply_all_effects(
                &source_high,
                &mut working_high,
                &sliders,
                &wheels,
                &pipeline,
                &color_mixer,
                preview_crop.is_cropped(actual_high_size),
            );
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
            let mut last_sliders = sliders;
            let mut last_wheels = wheels;
            let mut last_pipeline = pipeline.clone();
            let mut last_color_mixer = color_mixer.clone();
            let mut last_crop_area = crop_area;

            let mut is_dragging = false;
            let timeout = if is_proxy_enabled {
                Duration::from_millis(200)
            } else {
                // If proxy not enabled no need to wait for is_dragging state it doesn't matter
                // The timeout should be 0
                Duration::from_millis(0)
            };

            let frame_throttle = Duration::from_millis(16);
            let last_render_time = Instant::now();
            loop {
                let (mut sliders, mut wheels, mut pipeline, mut color_mixer, mut crop_area) =
                    if is_dragging {
                        match effect_rx.recv_timeout(timeout) {
                            Ok((s, w, p, m, c)) => (s, w, p, m, c),
                            Err(mpsc::RecvTimeoutError::Timeout) => {
                                is_dragging = false;
                                working_high = source_high.clone();
                                let preview_crop =
                                    last_crop_area.scaled(source_size, actual_high_size);
                                apply_all_effects(
                                    &source_high,
                                    &mut working_high,
                                    &last_sliders,
                                    &last_wheels,
                                    &last_pipeline,
                                    &last_color_mixer,
                                    preview_crop.is_cropped(actual_high_size),
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
                        match effect_rx.recv() {
                            Ok(g) => g,
                            Err(_) => break,
                        }
                    };

                while let Ok(newer) = effect_rx.try_recv() {
                    (sliders, wheels, pipeline, color_mixer, crop_area) = newer;
                }

                last_sliders = sliders;
                last_wheels = wheels;
                last_pipeline = pipeline.clone();
                last_color_mixer = color_mixer.clone();
                last_crop_area = crop_area;
                is_dragging = true;

                if last_render_time.elapsed() < frame_throttle {
                    continue;
                }

                if let (Some(sp), Some(wp)) = (source_proxy.as_ref(), working_proxy.as_mut()) {
                    let proxy_crop = crop_area.scaled(source_size, actual_proxy_size);
                    apply_all_effects(
                        sp,
                        wp,
                        &sliders,
                        &wheels,
                        &pipeline,
                        &color_mixer,
                        proxy_crop.is_cropped(actual_proxy_size),
                    );
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

        let sliders = self.slider_datas;
        let wheels = self.wheel_datas;
        let pipeline = self.pipeline.clone();
        let color_mixer = self.color_mixer.clone();
        let rotation = self.rotation;
        let crop_area = self.crop_area;
        let file_name = path.file_name().unwrap();
        let ext = path.extension().and_then(|e| e.to_str()).unwrap_or("png");
        export_path.push(file_name);
        export_path.set_extension(format!("output.{}", ext));

        let export_path_str = export_path.to_str().unwrap_or_default().to_string();
        let crop_region = self.crop_region;

        thread::spawn(move || {
            let result = (|| {
                let mut dyn_img = image::ImageReader::open(path)
                    .map_err(|e| format!("Failed to open: {e}"))?
                    .decode()
                    .map_err(|e| format!("Failed to decode: {e}"))?;
                if rotation != Orientation::NoTransforms {
                    dyn_img.apply_orientation(rotation);
                }
                if let Some(crop) = crop_region {
                    let cropped = dyn_img.crop(crop.x, crop.y, crop.width, crop.height);
                    dyn_img = cropped;
                }
                let source = dyn_img.to_rgba8();
                let mut export_image = source.clone();
                let export_size = (source.width(), source.height());
                apply_all_effects(
                    &source,
                    &mut export_image,
                    &sliders,
                    &wheels,
                    &pipeline,
                    &color_mixer,
                    crop_area.is_cropped(export_size),
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
    pub fn set_rotation(&mut self, rotation: Orientation) {
        self.rotation = rotation
    }

    pub fn reload(&mut self) {
        if let Some(path) = self.image_path.clone() {
            self.load_from_path(path);
        }
    }

    pub fn apply_effects(
        &mut self,
        slider_datas: SliderDatas,
        wheel_datas: WheelDatas,
        pipeline: Vec<ColorEffects>,
        color_mixer: Vec<ColorMixerPart>,
        crop_area: CropArea,
    ) {
        self.slider_datas = slider_datas;
        self.wheel_datas = wheel_datas;
        self.pipeline = pipeline.clone();
        self.color_mixer = color_mixer.clone();
        self.crop_area = crop_area;
        if let Some(ref tx) = self.effect_tx {
            let _ = tx.send((slider_datas, wheel_datas, pipeline, color_mixer, crop_area));
        }
    }
}
