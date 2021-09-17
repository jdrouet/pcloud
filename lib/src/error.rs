#[derive(Debug)]
pub enum Error {
    Payload(u16, String),
    Reqwest(reqwest::Error),
    Binary(crate::binary::Error),
    ResponseFormat,
    SerdeJson(serde_json::Error),
    Download(std::io::Error),
    Upload(std::io::Error),
}

impl Error {
    pub fn as_binary(&self) -> Option<&crate::binary::Error> {
        match self {
            Self::Binary(inner) => Some(&inner),
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
        log::error!("unable to execute command: {:?}", err);
        Self::Binary(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        log::error!("serialize issue: {:?}", err);
        Self::SerdeJson(err)
    }
}
