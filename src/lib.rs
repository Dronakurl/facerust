pub mod face_recognition;
pub mod ffi;
pub mod types;
pub mod watcher;

pub use face_recognition::FaceRecognition;
pub use types::{DbLoadStatus, DetectedFace, MatchResult, MatchResults};

// Re-export opencv for convenience
pub use opencv;

#[derive(Debug, thiserror::Error)]
pub enum FaceRecognitionError {
    #[error("OpenCV error: {0}")]
    OpenCv(#[from] opencv::Error),
    #[error("IO error: {0}")]
    Io(#[from] std::io::Error),
    #[error("Model not found: {0}")]
    ModelNotFound(String),
    #[error("Database not loaded")]
    DatabaseNotLoaded,
    #[error("Face detection failed")]
    DetectionFailed,
    #[error("Feature extraction failed")]
    FeatureExtractionFailed,
    #[error("Invalid image")]
    InvalidImage,
    #[error("Directory watch error: {0}")]
    WatchError(String),
}

pub type Result<T> = std::result::Result<T, FaceRecognitionError>;
