use thiserror::Error;

/// Error type for aspeak crate
#[derive(Error, Debug)]
#[non_exhaustive]
pub enum AspeakError {
    #[error("Websocket error")]
    WebSocketError(#[from] tokio_tungstenite::tungstenite::Error),
    #[error("Connection closed, code: {code}, reason: {reason}")]
    ConnectionCloseError { code: String, reason: String },
    #[error("Encountered invalid websocket message, invalid segment is: {0:?}")]
    InvalidWebSocketMessage(String),
    #[cfg(feature = "audio")]
    #[error("Audio decoder error")]
    DecoderError(#[from] rodio::decoder::DecoderError),
    #[cfg(feature = "audio")]
    #[error("Audio stream error")]
    StreamError(#[from] rodio::StreamError),
    #[cfg(feature = "audio")]
    #[error("Audio play error")]
    PlayError(#[from] rodio::PlayError),
    #[error("IO error")]
    IOError(#[from] std::io::Error),
    #[error("No input text/SSML.")]
    InputError,
    #[error("Failed to create SSML!")]
    XmlError(#[from] xml::writer::Error),
    #[error("{0}")]
    ArgumentError(String),
    #[error("Failed to parse url")]
    UrlParseError(#[from] url::ParseError),
    /// Other connection errors that are not covered by the above. (e.g. proxy error)
    #[error("Connection error: {0}")]
    GeneralConnectionError(String),
}

pub type Result<T> = std::result::Result<T, AspeakError>;

#[cfg(feature = "python")]
mod python {
    use super::AspeakError::{self, *};
    use color_eyre::eyre::Report;
    use pyo3::exceptions::{PyOSError, PyValueError};
    use pyo3::prelude::*;

    impl From<AspeakError> for PyErr {
        fn from(value: AspeakError) -> Self {
            match value {
                ArgumentError(detail) => PyValueError::new_err(detail),
                e => PyOSError::new_err(format!("{:?}", Report::from(e))),
            }
        }
    }
}
