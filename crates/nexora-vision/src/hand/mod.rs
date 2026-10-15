//! Suivi des mains : détecteur de paume + landmarks (ONNX), logique de pré/post-traitement
//! pure (testée sans matériel ni modèle). L'inférence `ort` est derrière la feature `onnx`.

pub mod anchors;
pub mod filter;
pub mod palm;
pub mod roi;

#[cfg(feature = "onnx")]
mod onnx;
#[cfg(feature = "onnx")]
pub use onnx::{default_models_dir, OnnxHandTracker};

#[cfg(all(feature = "onnx", feature = "camera"))]
mod live;
#[cfg(all(feature = "onnx", feature = "camera"))]
pub use live::CameraHandTracker;

use crate::Landmarks;

/// Main détectée, coordonnées normalisées 0..1 (image non miroir).
#[derive(Debug, Clone)]
pub struct HandResult {
    pub landmarks: Landmarks,
    /// Score de présence de la main (0..1).
    pub score: f32,
    /// Probabilité que ce soit la main droite (convention du modèle MediaPipe).
    pub right_prob: f32,
    pub roi: roi::Roi,
}
