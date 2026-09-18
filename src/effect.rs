use crate::ui::{color_mixer::ColorMixerPart, pipeline::ColorEffects};
use image::RgbaImage;
use rayon::prelude::*;

#[derive(Clone, Copy)]
pub struct SliderDatas {
    pub temperature: f32, // -100.0 to 100.0
    pub tint: f32,        // -100.0 to 100.0
    pub exposure: f32,    // -3.0 to 3.0
    pub contrast: f32,    // -100.0 to 100.0
    pub saturation: f32,  // 0.0 to 2.0
    pub hue_degrees: f32, // -180.0 to 180.0
}
impl SliderDatas {
    pub fn to_array(self) -> [f32; 6] {
        [
            self.temperature,
            self.tint,
            self.exposure,
            self.contrast,
            self.saturation,
            self.hue_degrees,
        ]
    }
}
impl Default for SliderDatas {
    fn default() -> Self {
        SliderDatas {
            temperature: 0.0,
            tint: 0.0,
            exposure: 0.0,
            contrast: 0.0,
            saturation: 1.0,
            hue_degrees: 0.0,
        }
    }
}
#[derive(Clone, Copy)]
pub struct WheelDatas {
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
impl WheelDatas {
    pub fn to_array(self) -> [[f32; 3]; 3] {
        [
            [self.lift_x, self.lift_y, self.lift_lum],
            [self.gamma_x, self.gamma_y, self.gamma_lum],
            [self.gain_x, self.gain_y, self.gain_lum],
        ]
    }
}
impl Default for WheelDatas {
    fn default() -> Self {
        WheelDatas {
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
    pub fn apply_effect(&self, r: &mut f32, g: &mut f32, b: &mut f32, p: &PrecomputedParams) {
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
pub fn apply_all_effects(
    source: &RgbaImage,
    working: &mut RgbaImage,
    slider_datas: &SliderDatas,
    wheel_datas: &WheelDatas,
    pipeline: &[ColorEffects],
    color_mixer: &[ColorMixerPart],
) {
    let radians = slider_datas.hue_degrees.to_radians();
    let cos_a = radians.cos();
    let sin_a = radians.sin();

    // Multiplier for the lift, gamma, gain wheels
    let intensity = 50.0;

    let params = PrecomputedParams {
        temperature: slider_datas.temperature,
        tint: slider_datas.tint,
        exp_mult: 2.0_f32.powf(slider_datas.exposure),
        cont_factor: (259.0 * (slider_datas.contrast + 255.0))
            / (255.0 * (259.0 - slider_datas.contrast)),
        hue_degrees: slider_datas.hue_degrees,
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
        saturation: slider_datas.saturation,
        // Lift (Shadows)
        lift_r: (wheel_datas.lift_y + wheel_datas.lift_x) * intensity + wheel_datas.lift_lum,
        lift_g: (wheel_datas.lift_y - wheel_datas.lift_x) * intensity + wheel_datas.lift_lum,
        lift_b: (wheel_datas.lift_y + wheel_datas.lift_x) * intensity + wheel_datas.lift_lum,
        // Gamma (Midtones)
        gamma_r: (wheel_datas.gamma_y + wheel_datas.gamma_x) * intensity + wheel_datas.gamma_lum,
        gamma_g: (wheel_datas.gamma_y - wheel_datas.gamma_x) * intensity + wheel_datas.gamma_lum,
        gamma_b: (-wheel_datas.gamma_y + wheel_datas.gamma_x) * intensity + wheel_datas.gamma_lum,
        // Gain (Highlights)
        gain_r: (wheel_datas.gain_y + wheel_datas.gain_x) * intensity + wheel_datas.gain_lum,
        gain_g: (wheel_datas.gain_y - wheel_datas.gain_x) * intensity + wheel_datas.gain_lum,
        gain_b: (-wheel_datas.gain_y + wheel_datas.gain_x) * intensity + wheel_datas.gain_lum,

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
                effect.apply_effect(&mut r, &mut g, &mut b, &params);
            }

            w_px[0] = r.clamp(0.0, 255.0) as u8;
            w_px[1] = g.clamp(0.0, 255.0) as u8;
            w_px[2] = b.clamp(0.0, 255.0) as u8;
            w_px[3] = s_px[3]; // alpha
        });
}
