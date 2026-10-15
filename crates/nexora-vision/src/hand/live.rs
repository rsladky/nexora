//! Caméra + suivi ONNX réunis derrière le trait `HandTracker` (features `camera` et `onnx`).

use std::path::Path;

use super::onnx::{default_models_dir, OnnxHandTracker};
use crate::camera::Camera;
use crate::{HandTracker, Landmarks, VisionError};

pub struct CameraHandTracker {
    camera: Camera,
    tracker: OnnxHandTracker,
}

impl CameraHandTracker {
    /// `index` : caméra (auto si `None`), `models_dir` : défaut `NEXORA_MODELS_DIR` ou `models/`.
    pub fn open(index: Option<u32>, models_dir: Option<&Path>) -> Result<Self, VisionError> {
        let camera = Camera::open(index, 640, 480, 30)?;
        let dir = models_dir.map_or_else(default_models_dir, Path::to_path_buf);
        Ok(Self {
            camera,
            tracker: OnnxHandTracker::new(&dir)?,
        })
    }

    pub fn camera(&self) -> &Camera {
        &self.camera
    }
}

impl HandTracker for CameraHandTracker {
    fn next(&mut self) -> Result<Option<Landmarks>, VisionError> {
        let frame = self.camera.frame()?;
        Ok(self.tracker.detect(&frame)?.map(|h| h.landmarks))
    }
}
