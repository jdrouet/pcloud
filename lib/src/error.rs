//! The errors thrown by the commands

/// All the possible errors returned by the clients and the API
#[derive(Debug)]
pub enum Error {
    /// Server side error, properly handled, returning a code and a message
    Protocol(u16, String),
    /// Error specific to the [`HttpClient`](crate::http::HttpClient)
    #[cfg(feature = "client-http")]
    Reqwest(reqwest::Error),
    /// Unable to read the response due to its format
    ResponseFormat,
    /// Unable to parse a JSON response
    SerdeJson(serde_json::Error),
    /// Error while downloading a file
    Download(std::io::Error),
    /// Error while uploading a file
    Upload(std::io::Error),
}

#[cfg(feature = "client-http")]
impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}

impl From<serde_json::Error> for Error {
    fn from(err: serde_json::Error) -> Self {
        Self::SerdeJson(err)
    }
}
