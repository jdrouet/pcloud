//! The errors thrown by the commands

/// All the possible errors returned by the clients and the API
#[derive(Debug)]
pub enum Error {
    /// Server side error, properly handled, returning a code and a message
    Protocol(u16, String),
    /// Error specific to the [`HttpClient`](crate::http::HttpClient)
    #[cfg(feature = "client-http")]
    Reqwest(reqwest::Error),
    /// Error specific to the [`BinaryClient`](crate::binary::BinaryClient)
    #[cfg(feature = "client-binary")]
    Binary(crate::binary::Error),
    /// Unable to read the response due to its format
    ResponseFormat,
    /// Unable to parse a JSON response
    SerdeJson(serde_json::Error),
    /// Error while downloading a file
    Download(std::io::Error),
    /// Error while uploading a file
    Upload(std::io::Error),
}

#[cfg(feature = "client-binary")]
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

#[cfg(feature = "client-http")]
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

#[cfg(feature = "client-binary")]
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
