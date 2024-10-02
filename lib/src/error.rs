//! The errors thrown by the commands

/// All the possible errors returned by the clients and the API
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Server side error, properly handled, returning a code and a message
    #[error("protocol error status {0}: {1}")]
    Protocol(u16, String),
    /// Error specific to the [`HttpClient`](crate::client::HttpClient)
    #[cfg(feature = "client-http")]
    #[error("network error")]
    Reqwest(
        #[from]
        #[source]
        reqwest::Error,
    ),
    /// Unable to parse a JSON response
    #[error("unable to decode pcloud response")]
    SerdeJson(
        #[from]
        #[source]
        serde_json::Error,
    ),
    /// Error while downloading a file
    #[error("unable to download file")]
    Download(#[source] std::io::Error),
    /// Error while uploading a file
    #[error("unable to upload file")]
    Upload(#[source] std::io::Error),
}
