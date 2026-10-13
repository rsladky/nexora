//! Backend `nokhwa` (Windows / macOS).

use nokhwa::pixel_format::RgbFormat;
use nokhwa::utils::{
    ApiBackend, CameraFormat, CameraIndex, FrameFormat, RequestedFormat, RequestedFormatType,
    Resolution,
};
use nokhwa::{query, Camera as NokhwaCamera};

use super::{err, CameraInfo, RgbFrame};
use crate::VisionError;
pub fn list_cameras() -> Result<Vec<CameraInfo>, VisionError> {
    let cams = query(ApiBackend::Auto).map_err(err)?;
    Ok(cams
        .into_iter()
        .filter_map(|c| match c.index() {
            CameraIndex::Index(i) => Some(CameraInfo {
                index: *i,
                name: c.human_name(),
            }),
            CameraIndex::String(_) => None,
        })
        .collect())
}

pub struct Camera {
    inner: NokhwaCamera,
    info: CameraInfo,
}

impl Camera {
    /// Ouvre une caméra et démarre le flux. `index = None` : première caméra qui fonctionne.
    /// La résolution et le format demandés sont indicatifs (« le plus proche » est retenu).
    pub fn open(
        index: Option<u32>,
        width: u32,
        height: u32,
        fps: u32,
    ) -> Result<Self, VisionError> {
        let candidates = match index {
            Some(i) => vec![CameraInfo {
                index: i,
                name: format!("caméra {i}"),
            }],
            None => list_cameras()?,
        };
        let mut last_err = VisionError::Camera("aucune caméra détectée".into());
        for info in candidates {
            match Self::try_open(&info, width, height, fps) {
                Ok(inner) => return Ok(Self { inner, info }),
                Err(e) => last_err = e,
            }
        }
        Err(last_err)
    }

    fn try_open(
        info: &CameraInfo,
        width: u32,
        height: u32,
        fps: u32,
    ) -> Result<NokhwaCamera, VisionError> {
        let wanted = CameraFormat::new(Resolution::new(width, height), FrameFormat::NV12, fps);
        let format = RequestedFormat::new::<RgbFormat>(RequestedFormatType::Closest(wanted));
        let mut cam = NokhwaCamera::new(CameraIndex::Index(info.index), format).map_err(err)?;
        cam.open_stream().map_err(err)?;
        Ok(cam)
    }

    pub fn info(&self) -> &CameraInfo {
        &self.info
    }

    /// Format effectivement négocié avec le pilote.
    pub fn format(&self) -> String {
        let f = self.inner.camera_format();
        format!(
            "{}x{} {:?} @ {} fps",
            f.width(),
            f.height(),
            f.format(),
            f.frame_rate()
        )
    }

    /// Attend la prochaine frame et la décode en RGB.
    pub fn frame(&mut self) -> Result<RgbFrame, VisionError> {
        let buf = self.inner.frame().map_err(err)?;
        let img = buf.decode_image::<RgbFormat>().map_err(err)?;
        Ok(RgbFrame {
            width: img.width(),
            height: img.height(),
            data: img.into_raw(),
        })
    }
}
