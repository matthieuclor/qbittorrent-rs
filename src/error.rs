use std::fmt;

#[derive(Debug)]
#[non_exhaustive]
pub enum Error {
    Network(Box<dyn std::error::Error + Send + Sync>),
    Timeout,
    Decode(serde_json::Error),
    Unauthorized,
    Api { status: u16, message: String },
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::Network(error) => write!(f, "qBittorrent request failed: {error}"),
            Self::Timeout => write!(f, "qBittorrent request timed out"),
            Self::Decode(error) => write!(f, "qBittorrent response could not be decoded: {error}"),
            Self::Unauthorized => write!(f, "qBittorrent rejected the API key"),
            Self::Api { status, message } => write!(f, "qBittorrent returned {status}: {message}"),
        }
    }
}

impl std::error::Error for Error {
    fn source(&self) -> Option<&(dyn std::error::Error + 'static)> {
        match self {
            Self::Network(error) => Some(error.as_ref()),
            Self::Decode(error) => Some(error),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(error: reqwest::Error) -> Self {
        if error.is_timeout() {
            Self::Timeout
        } else {
            Self::Network(Box::new(error))
        }
    }
}
