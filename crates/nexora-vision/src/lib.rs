//! Reconnaissance des gestes de la main.
//!
//! Pipeline cible : caméra (nokhwa) -> détection paume + 21 landmarks (ONNX via `ort`)
//! -> filtre One-Euro -> classifieur de geste (règles, puis MLP ONNX).
//! Cette version contient les types et le classifieur à règles, testables sans caméra.

use nexora_core::Gesture;

/// Les 21 points de la main (convention MediaPipe), coordonnées normalisées 0..1.
pub type Landmarks = [[f32; 3]; 21];

pub mod idx {
    pub const WRIST: usize = 0;
    pub const THUMB_TIP: usize = 4;
    pub const INDEX_MCP: usize = 5;
    pub const INDEX_TIP: usize = 8;
    pub const MIDDLE_MCP: usize = 9;
    pub const MIDDLE_TIP: usize = 12;
    pub const RING_TIP: usize = 16;
    pub const PINKY_TIP: usize = 20;
}

#[derive(Debug, thiserror::Error)]
pub enum VisionError {
    #[error("caméra indisponible : {0}")]
    Camera(String),
    #[error("modèle introuvable ou invalide : {0}")]
    Model(String),
}

/// Source de landmarks (caméra + modèle, ou mock en test).
pub trait HandTracker: Send {
    /// `None` si aucune main n'est visible sur la frame courante.
    fn next(&mut self) -> Result<Option<Landmarks>, VisionError>;
}

fn dist(a: [f32; 3], b: [f32; 3]) -> f32 {
    ((a[0] - b[0]).powi(2) + (a[1] - b[1]).powi(2)).sqrt()
}

/// Un doigt est « levé » si son bout est plus loin du poignet que sa base.
fn extended(l: &Landmarks, tip: usize, base: usize) -> bool {
    dist(l[tip], l[idx::WRIST]) > dist(l[base], l[idx::WRIST]) * 1.15
}

/// Classifieur à règles (baseline avant le MLP entraîné dans `ml/`).
pub fn classify(l: &Landmarks) -> Option<Gesture> {
    let scale = dist(l[idx::WRIST], l[idx::MIDDLE_MCP]).max(1e-6);
    let pinch_thr = 0.25 * scale;

    if dist(l[idx::THUMB_TIP], l[idx::INDEX_TIP]) < pinch_thr {
        return Some(Gesture::PinchIndex);
    }
    if dist(l[idx::THUMB_TIP], l[idx::MIDDLE_TIP]) < pinch_thr {
        return Some(Gesture::PinchMiddle);
    }

    let index = extended(l, idx::INDEX_TIP, idx::INDEX_MCP);
    let middle = extended(l, idx::MIDDLE_TIP, idx::MIDDLE_MCP);
    let ring = extended(l, idx::RING_TIP, 13);
    let pinky = extended(l, idx::PINKY_TIP, 17);

    match (index, middle, ring, pinky) {
        (true, false, false, false) => Some(Gesture::Point),
        (true, true, false, false) => Some(Gesture::TwoFingers),
        (true, true, true, true) => Some(Gesture::OpenPalm),
        (false, false, false, false) => Some(Gesture::Fist),
        _ => None,
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    /// Main synthétique : poignet en (0.5, 0.9), doigts vers le haut.
    fn hand(extended_fingers: [bool; 4]) -> Landmarks {
        let mut l = [[0.5, 0.9, 0.0]; 21];
        l[idx::WRIST] = [0.5, 0.9, 0.0];
        l[idx::THUMB_TIP] = [0.3, 0.7, 0.0];
        let bases = [
            (5usize, 8usize, 0.45f32),
            (9, 12, 0.5),
            (13, 16, 0.55),
            (17, 20, 0.6),
        ];
        for (i, (mcp, tip, x)) in bases.iter().enumerate() {
            l[*mcp] = [*x, 0.6, 0.0];
            l[*tip] = if extended_fingers[i] {
                [*x, 0.2, 0.0]
            } else {
                [*x, 0.65, 0.0]
            };
        }
        l
    }

    #[test]
    fn recognises_basic_poses() {
        assert_eq!(
            classify(&hand([true, false, false, false])),
            Some(Gesture::Point)
        );
        assert_eq!(
            classify(&hand([true, true, false, false])),
            Some(Gesture::TwoFingers)
        );
        assert_eq!(
            classify(&hand([true, true, true, true])),
            Some(Gesture::OpenPalm)
        );
        assert_eq!(
            classify(&hand([false, false, false, false])),
            Some(Gesture::Fist)
        );
    }

    #[test]
    fn recognises_pinch() {
        let mut l = hand([true, false, false, false]);
        l[idx::THUMB_TIP] = [l[idx::INDEX_TIP][0] + 0.01, l[idx::INDEX_TIP][1], 0.0];
        assert_eq!(classify(&l), Some(Gesture::PinchIndex));
    }
}
