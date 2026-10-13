//! Reconnaissance des gestes de la main.
//!
//! Pipeline cible : caméra (nokhwa) -> détection paume + 21 landmarks (ONNX via `ort`)
//! -> filtre One-Euro -> classifieur de geste (règles, puis MLP ONNX).
//! Cette version contient les types et le classifieur à règles, testables sans caméra.

use std::collections::VecDeque;

use nexora_core::{Event, Gesture};

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

/// Transforme les gestes bruts (un par frame) en événements `Start`/`End` stables.
/// Un geste n'est validé qu'après `threshold` frames consécutives identiques (debounce).
pub struct GestureTracker {
    threshold: usize,
    current: Option<Gesture>,
    candidate: Option<Gesture>,
    count: usize,
}

impl GestureTracker {
    pub fn new(threshold: usize) -> Self {
        Self {
            threshold: threshold.max(1),
            current: None,
            candidate: None,
            count: 0,
        }
    }

    /// Geste actuellement validé (stable).
    pub fn current(&self) -> Option<Gesture> {
        self.current
    }

    pub fn update(&mut self, raw: Option<Gesture>) -> Vec<Event> {
        if raw == self.candidate {
            self.count += 1;
        } else {
            self.candidate = raw;
            self.count = 1;
        }
        if self.count < self.threshold || self.candidate == self.current {
            return Vec::new();
        }
        let mut events = Vec::new();
        if let Some(old) = self.current {
            events.push(Event::GestureEnd(old));
        }
        if let Some(new) = self.candidate {
            events.push(Event::GestureStart(new));
        }
        self.current = self.candidate;
        events
    }
}

impl Default for GestureTracker {
    fn default() -> Self {
        Self::new(3)
    }
}

/// Mock de caméra : rejoue une séquence de frames puis ne voit plus aucune main.
pub struct ScriptedTracker {
    frames: VecDeque<Option<Landmarks>>,
}

impl ScriptedTracker {
    pub fn new(frames: impl IntoIterator<Item = Option<Landmarks>>) -> Self {
        Self {
            frames: frames.into_iter().collect(),
        }
    }
}

impl HandTracker for ScriptedTracker {
    fn next(&mut self) -> Result<Option<Landmarks>, VisionError> {
        Ok(self.frames.pop_front().flatten())
    }
}

/// Mains synthétiques pour les tests et les simulations (sans caméra).
pub mod synthetic {
    use super::{idx, Landmarks};

    /// Main synthétique : poignet en (0.5, 0.9), doigts vers le haut.
    /// `extended_fingers` : index, majeur, annulaire, auriculaire.
    pub fn hand(extended_fingers: [bool; 4]) -> Landmarks {
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

    /// Index levé dont le bout est placé en (x, y) (image non miroir, 0..1).
    pub fn pointing_at(x: f32, y: f32) -> Landmarks {
        let mut l = hand([true, false, false, false]);
        l[idx::INDEX_TIP] = [x, y, 0.0];
        l
    }

    /// Pincement pouce-index.
    pub fn pinch() -> Landmarks {
        let mut l = hand([true, false, false, false]);
        l[idx::THUMB_TIP] = [l[idx::INDEX_TIP][0] + 0.01, l[idx::INDEX_TIP][1], 0.0];
        l
    }
}

#[cfg(test)]
mod tests {
    use super::synthetic::{hand, pinch};
    use super::*;

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
        assert_eq!(classify(&pinch()), Some(Gesture::PinchIndex));
    }

    #[test]
    fn tracker_ignores_single_frame_noise() {
        let mut t = GestureTracker::new(3);
        let frames = [
            Some(Gesture::Fist),
            Some(Gesture::Fist),
            Some(Gesture::Point),
        ];
        for g in frames {
            assert!(t.update(g).is_empty());
        }
        assert_eq!(t.current(), None);
    }

    #[test]
    fn tracker_emits_start_then_end() {
        let mut t = GestureTracker::new(2);
        assert!(t.update(Some(Gesture::Fist)).is_empty());
        assert_eq!(
            t.update(Some(Gesture::Fist)),
            vec![Event::GestureStart(Gesture::Fist)]
        );
        // Stable : plus d'événement.
        assert!(t.update(Some(Gesture::Fist)).is_empty());
        assert!(t.update(None).is_empty());
        assert_eq!(t.update(None), vec![Event::GestureEnd(Gesture::Fist)]);
        assert_eq!(t.current(), None);
    }

    #[test]
    fn tracker_chains_end_and_start_on_change() {
        let mut t = GestureTracker::new(1);
        t.update(Some(Gesture::Fist));
        assert_eq!(
            t.update(Some(Gesture::OpenPalm)),
            vec![
                Event::GestureEnd(Gesture::Fist),
                Event::GestureStart(Gesture::OpenPalm)
            ]
        );
    }

    #[test]
    fn scripted_tracker_replays_then_runs_dry() {
        let mut t = ScriptedTracker::new([Some(hand([true; 4])), None]);
        assert!(t.next().unwrap().is_some());
        assert!(t.next().unwrap().is_none());
        assert!(t.next().unwrap().is_none());
    }
}
