#![warn(clippy::clone_on_ref_ptr, clippy::mod_module_files, clippy::todo)]

use glam::Vec2;
use serde::Serialize;
use thiserror::Error;

mod rect;
use rect::Rect;

#[derive(Error, Debug)]
pub enum YuNetError {
    #[error("Invalid input file")]
    InvalidFile,
    #[error("Face detection failed")]
    FaceDetectionFailed,
}

/// NOTE: "right" and "left" are defined in the natural face sense;
/// a person's right eye is seen on the left side of the screen.
///
/// Note that landmarks may occur outside of screen coordinates, as
/// YuNet can extrapolate their position from what's actually visible.
#[derive(Debug, Clone, Serialize)]
pub struct FaceLandmarks {
    pub right_eye: Vec2,
    pub left_eye: Vec2,
    pub nose: Vec2,
    pub mouth_right: Vec2,
    pub mouth_left: Vec2,
}

impl FaceLandmarks {
    fn from_yunet_landmark_array(landmarks: &[i32; 10]) -> Self {
        Self {
            right_eye: Vec2::new(landmarks[0] as f32, landmarks[1] as f32),
            left_eye: Vec2::new(landmarks[2] as f32, landmarks[3] as f32),
            nose: Vec2::new(landmarks[4] as f32, landmarks[5] as f32),
            mouth_right: Vec2::new(landmarks[6] as f32, landmarks[7] as f32),
            mouth_left: Vec2::new(landmarks[8] as f32, landmarks[9] as f32),
        }
    }
}

#[derive(Debug, Clone, Serialize)]
pub struct Face {
    /// How confident (0..1) YuNet is that the rectangle represents a valid face.
    confidence: f32,
    /// Location of the face on absolute pixel coordinates. This may fall outside
    /// of screen coordinates.
    rectangle: Rect,
    /// The resolution of the image in which this face was detected (width, height).
    detection_dimensions: (usize, usize),
    /// Coordinates of five face landmarks.
    landmarks: FaceLandmarks,
}

impl Face {
    /// Conversion is fallible, as YuNet has been known to report faces with
    /// negative dimensions, rarely.
    fn from_yunet_bridge_face(
        face_rect: &ffi::BridgeFace,
        detection_dimensions: (usize, usize),
    ) -> Self {
        Self {
            confidence: face_rect.score,
            rectangle: Rect::with_size(
                face_rect.x as f32,
                face_rect.y as f32,
                face_rect.w as f32,
                face_rect.h as f32,
            ),
            landmarks: FaceLandmarks::from_yunet_landmark_array(&face_rect.lm),
            detection_dimensions,
        }
    }

    /// How confident (0..1) YuNet is that the rectangle is a face.
    pub fn confidence(&self) -> f32 {
        self.confidence
    }

    /// Face rectangle in absolute pixel coordinates.
    pub fn rectangle(&self) -> Rect {
        self.rectangle
    }

    /// The minimum of normalized width and height.
    pub fn size(&self) -> f32 {
        let rect = self.normalized_rectangle();
        rect.w.min(rect.h)
    }

    /// Face rectangle in normalized 0..1 coordinates.
    pub fn normalized_rectangle(&self) -> Rect {
        Rect::with_size(
            self.rectangle.x / self.detection_dimensions.0 as f32,
            self.rectangle.y / self.detection_dimensions.1 as f32,
            self.rectangle.w / self.detection_dimensions.0 as f32,
            self.rectangle.h / self.detection_dimensions.1 as f32,
        )
    }

    /// Coordinates of five face landmarks.
    pub fn landmarks(&self) -> &FaceLandmarks {
        &self.landmarks
    }
}

pub fn detect_faces(bytes: &[u8], width: usize, height: usize) -> Result<Vec<Face>, YuNetError> {
    let faces = unsafe {
        crate::ffi::wrapper_detect_faces(
            bytes.as_ptr(),
            width as i32,
            height as i32,
            3 * width as i32,
        )
    };
    Ok(faces
        .into_iter()
        .map(|f| Face::from_yunet_bridge_face(&f, (width, height)))
        .collect())
}

#[cxx::bridge]
mod ffi {
    // Shared type visible from both C++ and Rust
    #[derive(Debug)]
    struct BridgeFace {
        score: f32,
        x: i32,
        y: i32,
        w: i32,
        h: i32,
        lm: [i32; 10],
    }

    unsafe extern "C++" {
        include!("rusty-yunet/src/bridge_wrapper.h");

        unsafe fn wrapper_detect_faces(
            rgb_image_data: *const u8,
            width: i32,
            height: i32,
            step: i32,
        ) -> Vec<BridgeFace>;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn detect_sample_faces() {
        // Loads a sample with three faces clearly staggered in distance. Detecting the biggest
        // face with high confidence should be completely expected. Detecting the mid-sized face
        // is good, as it probably stretches what we consider "presence" in front of a normal
        // installation. Detecting the smallest face is very unrealistic and unnecessary.
        //
        // Detecting two faces with this test at this resolution can be considered a good result.
        let image = image::open("sample.jpg").unwrap();
        let bytes = image.to_bgr8().to_vec();
        let faces = detect_faces(
            &bytes,
            image::GenericImageView::width(&image) as usize,
            image::GenericImageView::height(&image) as usize,
        )
        .unwrap();
        assert_eq!(2, faces.len());
    }
}
