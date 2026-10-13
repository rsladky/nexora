//! Backend Linux : V4L2 via le crate `v4l` (flux mmap).

use v4l::buffer::Type;
use v4l::capability::Flags;
use v4l::context::enum_devices;
use v4l::io::mmap::Stream;
use v4l::io::traits::CaptureStream;
use v4l::video::Capture;
use v4l::{Device, FourCC};

use super::{err, nv12_to_rgb, yuyv_to_rgb, CameraInfo, RgbFrame};
use crate::VisionError;

/// Caméras « capture simple plan » (exclut les décodeurs matériels type `avd`).
pub fn list_cameras() -> Result<Vec<CameraInfo>, VisionError> {
    let mut cams: Vec<CameraInfo> = enum_devices()
        .into_iter()
        .filter(|n| {
            Device::with_path(n.path())
                .and_then(|d| d.query_caps())
                .map(|c| c.capabilities.contains(Flags::VIDEO_CAPTURE))
                .unwrap_or(false)
        })
        .map(|n| CameraInfo {
            index: n.index() as u32,
            name: n.name().unwrap_or_else(|| format!("caméra {}", n.index())),
        })
        .collect();
    cams.sort_by_key(|c| c.index);
    Ok(cams)
}

pub struct Camera {
    stream: Stream<'static>,
    info: CameraInfo,
    width: u32,
    height: u32,
    stride: u32,
    fourcc: FourCC,
}

impl Camera {
    /// Ouvre une caméra et démarre le flux. `index = None` : première caméra qui fonctionne.
    /// La résolution est indicative (le pilote choisit la plus proche). `_fps` : débit par défaut.
    pub fn open(
        index: Option<u32>,
        width: u32,
        height: u32,
        _fps: u32,
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
            match Self::try_open(info, width, height) {
                Ok(cam) => return Ok(cam),
                Err(e) => last_err = e,
            }
        }
        Err(last_err)
    }

    fn try_open(info: CameraInfo, width: u32, height: u32) -> Result<Self, VisionError> {
        let dev = Device::new(info.index as usize).map_err(err)?;
        let mut last = VisionError::Camera("aucun format supporté (NV12, YUYV)".into());
        for fourcc in [b"NV12", b"YUYV"] {
            let mut fmt = dev.format().map_err(err)?;
            fmt.width = width;
            fmt.height = height;
            fmt.fourcc = FourCC::new(fourcc);
            match dev.set_format(&fmt) {
                // Le pilote peut substituer un autre format : on ne garde que ceux décodés ici.
                Ok(got) if got.fourcc == FourCC::new(fourcc) => {
                    let stream = Stream::with_buffers(&dev, Type::VideoCapture, 4).map_err(err)?;
                    return Ok(Self {
                        stream,
                        info,
                        width: got.width,
                        height: got.height,
                        stride: got.stride,
                        fourcc: got.fourcc,
                    });
                }
                Ok(got) => last = VisionError::Camera(format!("format {} non géré", got.fourcc)),
                Err(e) => last = err(e),
            }
        }
        Err(last)
    }

    pub fn info(&self) -> &CameraInfo {
        &self.info
    }

    /// Format effectivement négocié avec le pilote.
    pub fn format(&self) -> String {
        format!("{}x{} {}", self.width, self.height, self.fourcc)
    }

    /// Attend la prochaine frame et la décode en RGB.
    pub fn frame(&mut self) -> Result<RgbFrame, VisionError> {
        let (buf, _meta) = self.stream.next().map_err(err)?;
        let frame = if self.fourcc == FourCC::new(b"NV12") {
            nv12_to_rgb(buf, self.width, self.height, self.stride)
        } else {
            yuyv_to_rgb(buf, self.width, self.height, self.stride)
        };
        frame.ok_or_else(|| VisionError::Camera("frame incomplète".into()))
    }
}
