//! Régions d'intérêt rotatées, letterbox et warp bilinéaire (sans dépendance image).

use std::f32::consts::{PI, TAU};

use super::palm::PalmDetection;
use crate::{Landmarks, RgbFrame};

/// Carré rotaté en pixels image. `rotation` > 0 : le haut du crop pointe vers la droite.
#[derive(Debug, Clone, Copy, PartialEq)]
pub struct Roi {
    pub cx: f32,
    pub cy: f32,
    pub size: f32,
    pub rotation: f32,
}

fn normalize_angle(a: f32) -> f32 {
    let mut a = a;
    while a > PI {
        a -= TAU;
    }
    while a < -PI {
        a += TAU;
    }
    a
}

/// Rotation qui redresse le vecteur `from -> to` (en pixels image) à la verticale.
fn upright_rotation(from: [f32; 2], to: [f32; 2]) -> f32 {
    normalize_angle(PI / 2.0 - (-(to[1] - from[1])).atan2(to[0] - from[0]))
}

impl Roi {
    /// Image pixel d'un point du crop, `(u, v)` dans 0..1.
    pub fn to_image(&self, u: f32, v: f32) -> [f32; 2] {
        let (tx, ty) = (u - 0.5, v - 0.5);
        let (s, c) = self.rotation.sin_cos();
        [
            self.cx + self.size * (tx * c - ty * s),
            self.cy + self.size * (tx * s + ty * c),
        ]
    }

    /// Inverse de [`Roi::to_image`] : coordonnées 0..1 dans le crop.
    pub fn from_image(&self, x: f32, y: f32) -> [f32; 2] {
        let (dx, dy) = ((x - self.cx) / self.size, (y - self.cy) / self.size);
        let (s, c) = self.rotation.sin_cos();
        [dx * c + dy * s + 0.5, -dx * s + dy * c + 0.5]
    }

    /// ROI depuis une paume (poignet = kp 0, base du majeur = kp 2) : décalage -0.5, échelle 2.6.
    pub fn from_palm(p: &PalmDetection, lb: &Letterbox) -> Self {
        let kp = |i: usize| lb.to_image(p.keypoints[i][0], p.keypoints[i][1]);
        let rotation = upright_rotation(kp(0), kp(2));
        let c = lb.to_image(p.cx, p.cy);
        let (w, h) = (p.w * lb.input / lb.scale, p.h * lb.input / lb.scale);
        let (s, co) = rotation.sin_cos();
        Self {
            cx: c[0] + 0.5 * h * s,
            cy: c[1] - 0.5 * h * co,
            size: w.max(h) * 2.6,
            rotation,
        }
    }

    /// ROI de suivi depuis les landmarks de la frame précédente (en pixels image) :
    /// rotation poignet -> base du majeur, échelle 2.0, décalage -0.1.
    pub fn from_landmarks(px: &[[f32; 2]; 21]) -> Self {
        let rotation = upright_rotation(px[0], px[9]);
        let (s, c) = rotation.sin_cos();
        let subset = [0usize, 1, 2, 3, 5, 6, 9, 10, 13, 14, 17, 18];
        // Boîte englobante dans le repère redressé.
        let (mut x0, mut y0, mut x1, mut y1) = (f32::MAX, f32::MAX, f32::MIN, f32::MIN);
        for &i in &subset {
            let (x, y) = (px[i][0], px[i][1]);
            let (rx, ry) = (x * c - y * s, x * s + y * c);
            x0 = x0.min(rx);
            x1 = x1.max(rx);
            y0 = y0.min(ry);
            y1 = y1.max(ry);
        }
        let (mx, my) = ((x0 + x1) / 2.0, (y0 + y1) / 2.0);
        let (h, w) = (y1 - y0, x1 - x0);
        // Retour au repère image (rotation inverse).
        let (cx, cy) = (mx * c + my * s, -mx * s + my * c);
        let size = w.max(h) * 2.0;
        Self {
            cx: cx + 0.1 * size * s,
            cy: cy - 0.1 * size * c,
            size,
            rotation,
        }
    }

    /// Landmarks du crop (x, y en pixels du crop `crop`, z) -> coordonnées image normalisées.
    pub fn landmarks_to_image(&self, raw: &[f32], crop: f32, width: u32, height: u32) -> Landmarks {
        let mut out = [[0.0; 3]; 21];
        for (i, p) in out.iter_mut().enumerate() {
            let [x, y] = self.to_image(raw[3 * i] / crop, raw[3 * i + 1] / crop);
            *p = [
                x / width as f32,
                y / height as f32,
                raw[3 * i + 2] / crop * self.size / width as f32,
            ];
        }
        out
    }
}

/// Redimensionnement avec bandes (letterbox) vers un carré `input` x `input`.
#[derive(Debug, Clone, Copy)]
pub struct Letterbox {
    pub input: f32,
    pub scale: f32,
    pub pad_x: f32,
    pub pad_y: f32,
}

impl Letterbox {
    /// Point 0..1 de l'image letterboxée -> pixel de l'image source.
    pub fn to_image(&self, x: f32, y: f32) -> [f32; 2] {
        [
            (x * self.input - self.pad_x) / self.scale,
            (y * self.input - self.pad_y) / self.scale,
        ]
    }
}

fn pixel(frame: &RgbFrame, x: i64, y: i64) -> [f32; 3] {
    let x = x.clamp(0, frame.width as i64 - 1) as usize;
    let y = y.clamp(0, frame.height as i64 - 1) as usize;
    let i = (y * frame.width as usize + x) * 3;
    [
        frame.data[i] as f32,
        frame.data[i + 1] as f32,
        frame.data[i + 2] as f32,
    ]
}

/// Échantillon bilinéaire (bords répliqués), centres de pixels en (i + 0.5).
fn sample(frame: &RgbFrame, x: f32, y: f32) -> [f32; 3] {
    let (fx, fy) = (x - 0.5, y - 0.5);
    let (x0, y0) = (fx.floor(), fy.floor());
    let (ax, ay) = (fx - x0, fy - y0);
    let (x0, y0) = (x0 as i64, y0 as i64);
    let (p00, p10) = (pixel(frame, x0, y0), pixel(frame, x0 + 1, y0));
    let (p01, p11) = (pixel(frame, x0, y0 + 1), pixel(frame, x0 + 1, y0 + 1));
    let mut out = [0.0; 3];
    for c in 0..3 {
        let top = p00[c] * (1.0 - ax) + p10[c] * ax;
        let bot = p01[c] * (1.0 - ax) + p11[c] * ax;
        out[c] = (top * (1.0 - ay) + bot * ay) / 255.0;
    }
    out
}

/// Image -> tenseur NHWC `input x input x 3` en 0..1, avec bandes noires.
pub fn letterbox(frame: &RgbFrame, input: usize) -> (Vec<f32>, Letterbox) {
    let scale = input as f32 / frame.width.max(frame.height) as f32;
    let (nw, nh) = (frame.width as f32 * scale, frame.height as f32 * scale);
    let lb = Letterbox {
        input: input as f32,
        scale,
        pad_x: (input as f32 - nw) / 2.0,
        pad_y: (input as f32 - nh) / 2.0,
    };
    let mut out = vec![0.0; input * input * 3];
    for v in 0..input {
        for u in 0..input {
            let (sx, sy) = (
                (u as f32 + 0.5 - lb.pad_x) / scale,
                (v as f32 + 0.5 - lb.pad_y) / scale,
            );
            if sx < 0.0 || sy < 0.0 || sx > frame.width as f32 || sy > frame.height as f32 {
                continue;
            }
            let i = (v * input + u) * 3;
            out[i..i + 3].copy_from_slice(&sample(frame, sx, sy));
        }
    }
    (out, lb)
}

/// Extrait la ROI rotatée en tenseur NHWC `size x size x 3` (0..1).
pub fn warp(frame: &RgbFrame, roi: &Roi, size: usize) -> Vec<f32> {
    let mut out = vec![0.0; size * size * 3];
    for v in 0..size {
        for u in 0..size {
            let [x, y] = roi.to_image(
                (u as f32 + 0.5) / size as f32,
                (v as f32 + 0.5) / size as f32,
            );
            let i = (v * size + u) * 3;
            out[i..i + 3].copy_from_slice(&sample(frame, x, y));
        }
    }
    out
}

#[cfg(test)]
mod tests {
    use super::*;

    fn frame(w: u32, h: u32, f: impl Fn(u32, u32) -> u8) -> RgbFrame {
        let mut data = Vec::new();
        for y in 0..h {
            for x in 0..w {
                data.extend_from_slice(&[f(x, y); 3]);
            }
        }
        RgbFrame {
            width: w,
            height: h,
            data,
        }
    }

    #[test]
    fn roi_roundtrip() {
        let roi = Roi {
            cx: 300.0,
            cy: 200.0,
            size: 150.0,
            rotation: 0.7,
        };
        let p = roi.to_image(0.2, 0.9);
        let q = roi.from_image(p[0], p[1]);
        assert!((q[0] - 0.2).abs() < 1e-4 && (q[1] - 0.9).abs() < 1e-4);
    }

    #[test]
    fn upright_hand_has_zero_rotation() {
        assert!(upright_rotation([100.0, 300.0], [100.0, 100.0]).abs() < 1e-6);
        // Main inclinée vers la droite : rotation positive.
        assert!(upright_rotation([100.0, 300.0], [200.0, 100.0]) > 0.0);
    }

    #[test]
    fn warp_uniform_and_identity() {
        let f = frame(64, 64, |_, _| 128);
        let roi = Roi {
            cx: 32.0,
            cy: 32.0,
            size: 40.0,
            rotation: 0.4,
        };
        assert!(warp(&f, &roi, 8)
            .iter()
            .all(|v| (v - 128.0 / 255.0).abs() < 1e-5));

        // Dégradé horizontal : ROI = image entière, sans rotation -> identité.
        let g = frame(32, 32, |x, _| (x * 8) as u8);
        let full = Roi {
            cx: 16.0,
            cy: 16.0,
            size: 32.0,
            rotation: 0.0,
        };
        let out = warp(&g, &full, 32);
        for x in 0..32 {
            assert!((out[x * 3] - (x * 8) as f32 / 255.0).abs() < 1e-5);
        }
    }

    #[test]
    fn letterbox_maps_back() {
        let f = frame(640, 480, |_, _| 200);
        let (t, lb) = letterbox(&f, 192);
        assert_eq!(t.len(), 192 * 192 * 3);
        // Bande noire en haut, image au centre.
        assert_eq!(t[0], 0.0);
        assert!((t[(96 * 192 + 96) * 3] - 200.0 / 255.0).abs() < 1e-5);
        let p = lb.to_image(0.5, 0.5);
        assert!((p[0] - 320.0).abs() < 1e-3 && (p[1] - 240.0).abs() < 1e-3);
    }

    #[test]
    fn tracking_roi_from_upright_hand() {
        let mut px = [[0.0; 2]; 21];
        for (i, p) in px.iter_mut().enumerate() {
            *p = [100.0 + (i % 5) as f32 * 10.0, 300.0 - (i as f32) * 8.0];
        }
        px[0] = [120.0, 320.0];
        px[9] = [120.0, 200.0];
        let roi = Roi::from_landmarks(&px);
        assert!(roi.rotation.abs() < 1e-5);
        assert!(roi.size > 0.0);
    }
}
