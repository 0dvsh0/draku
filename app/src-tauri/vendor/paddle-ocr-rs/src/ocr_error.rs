use thiserror::Error;

#[derive(Error, Debug)]
pub enum OcrError {
    #[error("Ort error: {0}")]
    Ort(String),
    #[error("Io error")]
    Io(#[from] std::io::Error),
    #[error("Image error")]
    ImageError(#[from] image::ImageError),
    #[error("Session not initialized")]
    SessionNotInitialized,
}

// `ort::Error` is generic over the recoverable builder type in newer
// releases. Store the human message so one conversion covers all of them.
impl<T> From<ort::Error<T>> for OcrError {
    fn from(e: ort::Error<T>) -> Self {
        OcrError::Ort(e.to_string())
    }
}
