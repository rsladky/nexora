//! Capture webcam réelle (feature `camera`).
//!
//! - Linux : `v4l` directement (NV12 et YUYV). `nokhwa` refuse le pilote `apple-isp`
//!   (Asahi) car il rejette les intervalles non réduits (`256/7680` au lieu de `1/30`).
//! - Autres OS (Windows) : `nokhwa`.
//!
//! Sans index explicite, on prend la première caméra qui ouvre un flux.

use crate::VisionError;

#[cfg(target_os = "linux")]
mod linux;
#[cfg(target_os = "linux")]
use linux as backend;

#[cfg(not(target_os = "linux"))]
mod nokhwa_backend;
#[cfg(not(target_os = "linux"))]
use nokhwa_backend as backend;

pub use backend::{list_cameras, Camera};

pub use crate::RgbFrame;

#[derive(Debug, Clone)]
pub struct CameraInfo {
    pub index: u32,
    pub name: String,
}

fn err(e: impl std::fmt::Display) -> VisionError {
    VisionError::Camera(e.to_string())
}

/// BT.709 limité (16..235) -> RGB.
fn yuv_to_rgb(y: u8, u: u8, v: u8) -> [u8; 3] {
    let y = 1.164 * (y as f32 - 16.0);
    let (u, v) = (u as f32 - 128.0, v as f32 - 128.0);
    let c = |x: f32| x.round().clamp(0.0, 255.0) as u8;
    [
        c(y + 1.793 * v),
        c(y - 0.213 * u - 0.533 * v),
        c(y + 2.112 * u),
    ]
}

/// NV12 : plan Y (`stride` octets par ligne) suivi d'un plan UV entrelacé demi-résolution.
fn nv12_to_rgb(data: &[u8], width: u32, height: u32, stride: u32) -> Option<RgbFrame> {
    let (w, h, s) = (width as usize, height as usize, stride as usize);
    if data.len() < s * h * 3 / 2 {
        return None;
    }
    let (luma, chroma) = data.split_at(s * h);
    let mut out = Vec::with_capacity(w * h * 3);
    for row in 0..h {
        for col in 0..w {
            let uv = (row / 2) * s + (col & !1);
            out.extend_from_slice(&yuv_to_rgb(luma[row * s + col], chroma[uv], chroma[uv + 1]));
        }
    }
    Some(RgbFrame {
        width,
        height,
        data: out,
    })
}

/// YUYV (YUY2) : 2 pixels par paquet Y0 U Y1 V.
fn yuyv_to_rgb(data: &[u8], width: u32, height: u32, stride: u32) -> Option<RgbFrame> {
    let (w, h, s) = (width as usize, height as usize, stride as usize);
    if data.len() < s * h {
        return None;
    }
    let mut out = Vec::with_capacity(w * h * 3);
    for row in 0..h {
        let line = &data[row * s..row * s + w * 2];
        for px in line.as_chunks::<4>().0 {
            out.extend_from_slice(&yuv_to_rgb(px[0], px[1], px[3]));
            out.extend_from_slice(&yuv_to_rgb(px[2], px[1], px[3]));
        }
    }
    Some(RgbFrame {
        width,
        height,
        data: out,
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn nv12_grey_and_size() {
        // 4x2 : Y=126 (gris moyen), U=V=128 (neutre).
        let mut data = vec![126u8; 8];
        data.extend_from_slice(&[128; 4]);
        let f = nv12_to_rgb(&data, 4, 2, 4).unwrap();
        assert_eq!(f.data.len(), 4 * 2 * 3);
        assert!(f.data.chunks(3).all(|p| p[0] == p[1] && p[1] == p[2]));
        assert!(nv12_to_rgb(&data[..5], 4, 2, 4).is_none());
    }

    #[test]
    fn yuyv_neutral_chroma_is_grey() {
        let data = [235, 128, 235, 128, 16, 128, 16, 128];
        let f = yuyv_to_rgb(&data, 4, 1, 8).unwrap();
        assert_eq!(&f.data[..3], &[255, 255, 255]);
        assert_eq!(&f.data[9..12], &[0, 0, 0]);
    }
}
