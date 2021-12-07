#[derive(Debug)]
pub enum Error {
    Protocol(u16, String),
    Reqwest(reqwest::Error),
    Binary(crate::binary::Error),
    ResponseFormat,
    SerdeJson(serde_json::Error),
    Download(std::io::Error),
    Upload(std::io::Error),
}

impl Error {
    pub fn is_binary(&self) -> bool {
        matches!(self, Self::Binary(_))
    }

    pub fn as_binary(&self) -> Option<&crate::binary::Error> {
        match self {
            Self::Binary(value) => Some(value),
            _ => None,
        }
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

impl From<crate::binary::Error> for Error {
    fn from(err: crate::binary::Error) -> Self {
        Self::Binary(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJson(err)
    }
}
