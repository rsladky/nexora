//! Inférence ONNX : détecteur de paume puis modèle de landmarks (feature `onnx`).

use std::path::{Path, PathBuf};
use std::time::Instant;

use ort::environment::Environment;
use ort::session::Session;
use ort::value::Tensor;

use super::anchors::{generate_anchors, PALM_INPUT};
use super::filter::LandmarkFilter;
use super::palm::{decode, weighted_nms};
use super::roi::{letterbox, warp, Roi};
use super::HandResult;
use crate::{Landmarks, RgbFrame, VisionError};

const LANDMARK_INPUT: usize = 224;
const PALM_THRESHOLD: f32 = 0.5;
const PRESENCE_THRESHOLD: f32 = 0.5;

fn model_err(e: impl std::fmt::Display) -> VisionError {
    VisionError::Model(e.to_string())
}

/// Dossier des modèles : `NEXORA_MODELS_DIR`, sinon `models/` à la racine du workspace.
pub fn default_models_dir() -> PathBuf {
    std::env::var_os("NEXORA_MODELS_DIR")
        .map(PathBuf::from)
        .unwrap_or_else(|| Path::new(env!("CARGO_MANIFEST_DIR")).join("../../models"))
}

pub struct OnnxHandTracker {
    _env: Environment,
    palm: Session,
    landmark: Session,
    anchors: Vec<[f32; 2]>,
    filter: LandmarkFilter,
    /// ROI de suivi issue de la frame précédente (évite de relancer la détection de paume).
    tracked: Option<Roi>,
    last: Option<Instant>,
}

impl OnnxHandTracker {
    pub fn new(models_dir: &Path) -> Result<Self, VisionError> {
        let env = ort::init().build().map_err(model_err)?;
        let load = |name: &str| -> Result<Session, VisionError> {
            let path = models_dir.join(name);
            Session::builder(&env)
                .map_err(model_err)?
                .with_intra_threads(2)
                .map_err(model_err)?
                .commit_from_file(&path)
                .map_err(|e| model_err(format!("{} : {e}", path.display())))
        };
        let (palm, landmark) = (load("palm_detection.onnx")?, load("hand_landmark.onnx")?);
        Ok(Self {
            _env: env,
            palm,
            landmark,
            anchors: generate_anchors(),
            filter: LandmarkFilter::default(),
            tracked: None,
            last: None,
        })
    }

    /// Cherche une paume dans la frame et en déduit une ROI.
    fn detect_palm(&mut self, frame: &RgbFrame) -> Result<Option<Roi>, VisionError> {
        let (input, lb) = letterbox(frame, PALM_INPUT);
        let tensor =
            Tensor::from_array(([1usize, PALM_INPUT, PALM_INPUT, 3], input)).map_err(model_err)?;
        let outputs = self.palm.run(ort::inputs![tensor]).map_err(model_err)?;
        let (_, regressors) = outputs[0].try_extract_tensor::<f32>().map_err(model_err)?;
        let (_, scores) = outputs[1].try_extract_tensor::<f32>().map_err(model_err)?;
        let dets = weighted_nms(
            decode(regressors, scores, &self.anchors, PALM_THRESHOLD),
            0.3,
        );
        Ok(dets.first().map(|p| Roi::from_palm(p, &lb)))
    }

    /// Traite une frame. `None` si aucune main n'est visible.
    pub fn detect(&mut self, frame: &RgbFrame) -> Result<Option<HandResult>, VisionError> {
        let now = Instant::now();
        let dt = self
            .last
            .map_or(1.0 / 30.0, |t| now.duration_since(t).as_secs_f32());
        self.last = Some(now);

        let roi = match self.tracked {
            Some(r) => r,
            None => match self.detect_palm(frame)? {
                Some(r) => r,
                None => return Ok(None),
            },
        };

        let input = warp(frame, &roi, LANDMARK_INPUT);
        let tensor = Tensor::from_array(([1usize, LANDMARK_INPUT, LANDMARK_INPUT, 3], input))
            .map_err(model_err)?;
        let outputs = self.landmark.run(ort::inputs![tensor]).map_err(model_err)?;
        let (_, raw) = outputs[0].try_extract_tensor::<f32>().map_err(model_err)?;
        let (_, presence) = outputs[1].try_extract_tensor::<f32>().map_err(model_err)?;
        let (_, handed) = outputs[2].try_extract_tensor::<f32>().map_err(model_err)?;
        let (score, right_prob) = (presence[0], handed[0]);

        if score < PRESENCE_THRESHOLD {
            self.tracked = None;
            self.filter.reset();
            return Ok(None);
        }

        let mut landmarks: Landmarks =
            roi.landmarks_to_image(raw, LANDMARK_INPUT as f32, frame.width, frame.height);
        let px = landmarks.map(|p| [p[0] * frame.width as f32, p[1] * frame.height as f32]);
        self.tracked = Some(Roi::from_landmarks(&px));
        self.filter.apply(&mut landmarks, dt);

        Ok(Some(HandResult {
            landmarks,
            score,
            right_prob,
            roi,
        }))
    }
}
