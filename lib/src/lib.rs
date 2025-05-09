#![doc = include_str!("../readme.md")]

use std::borrow::Cow;

// Module responsible for building requests to the API, including setting parameters and
// configuring request details such as method type, headers, and body content.
pub mod builder;

// Module for handling entries in the system. This could include creating, modifying,
// or retrieving data related to various types of entries (e.g., file or folder entries).
pub mod entry;

// Module for dealing with files, including operations like file uploads, downloads,
// file metadata retrieval, and manipulation.
pub mod file;

// Module for handling folder-related operations such as creating, listing,
// or manipulating folders in the system.
pub mod folder;

/// Module handling general operations
/// https://docs.pcloud.com/methods/general/
pub mod general;

// Module for working with streams, likely including streaming files or media
// content, such as audio and video, over the network or from storage.
pub mod stream;

// Private module responsible for date manipulation, likely for handling timestamps
// or other date-related utilities across the library.
mod date;

// Private module that contains the logic for handling HTTP requests, such as sending GET, POST,
// PUT requests, serializing parameters, and processing responses from the API.
mod request;

// Re-exporting the reqwest crate for convenient access
pub use reqwest;

/// The default user agent used by the HTTP client, derived from crate name and version.
const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

/// Base URL for the EU region.
pub const EU_REGION: &str = "https://eapi.pcloud.com";

/// Base URL for the US region.
pub const US_REGION: &str = "https://api.pcloud.com";

/// Represents a pCloud API region.
#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum Region {
    /// Europe region endpoint
    #[serde(alias = "EU")]
    Eu,
    /// United States region endpoint
    #[serde(alias = "US")]
    Us,
}

impl Region {
    /// Returns the base URL associated with the selected region.
    const fn base_url(&self) -> &'static str {
        match self {
            Self::Eu => EU_REGION,
            Self::Us => US_REGION,
        }
    }
}

impl Region {
    /// Attempts to create a `Region` from the `PCLOUD_REGION` environment variable.
    ///
    /// Recognizes `"eu"`, `"EU"`, `"us"`, and `"US"` as valid inputs.
    pub fn from_env() -> Option<Self> {
        let name = std::env::var("PCLOUD_REGION").ok()?;
        match name.as_str() {
            "eu" | "EU" => Some(Self::Eu),
            "us" | "US" => Some(Self::Us),
            _ => None,
        }
    }
}

/// Authentication credentials used for pCloud API requests.
#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Credentials {
    /// Uses a personal access token.
    AccessToken { access_token: String },
    /// Uses a username and password for authentication.
    UsernamePassword { username: String, password: String },
}

impl Credentials {
    /// Creates credentials using an access token.
    pub fn access_token(value: impl Into<String>) -> Self {
        Self::AccessToken {
            access_token: value.into(),
        }
    }

    /// Creates credentials using a username and password.
    pub fn username_password(username: impl Into<String>, password: impl Into<String>) -> Self {
        Self::UsernamePassword {
            username: username.into(),
            password: password.into(),
        }
    }
}

impl Credentials {
    /// Creates a credential based on the environment variables
    ///
    /// When `PCLOUD_ACCESS_TOKEN` is set, a `Some(Credentials::AccessToken)` will be created.
    ///
    /// When `PCLOUD_USERNAME` and `PCLOUD_PASSWORD` are set, a `Some(Credentials::UsernamePassword)` will be created.
    ///
    /// If none are set, `None` is returned.
    ///
    /// ```rust
    /// use pcloud::Credentials;
    ///
    /// match Credentials::from_env() {
    ///     Some(Credentials::AccessToken { .. }) => println!("uses an access token"),
    ///     Some(Credentials::UsernamePassword { .. }) => println!("uses a username and a password"),
    ///     None => eprintln!("no credentials provided"),
    /// }
    /// ```
    pub fn from_env() -> Option<Self> {
        if let Ok(access_token) = std::env::var("PCLOUD_ACCESS_TOKEN") {
            Some(Self::AccessToken { access_token })
        } else if let (Ok(username), Ok(password)) = (
            std::env::var("PCLOUD_USERNAME"),
            std::env::var("PCLOUD_PASSWORD"),
        ) {
            Some(Self::UsernamePassword { username, password })
        } else {
            None
        }
    }
}

impl std::fmt::Debug for Credentials {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct(stringify!(Credentials))
            .finish_non_exhaustive()
    }
}

/// HTTP client used to interact with the pCloud API.
#[derive(Debug)]
pub struct Client {
    base_url: Cow<'static, str>,
    credentials: Credentials,
    inner: reqwest::Client,
}

impl Client {
    /// Creates a new `ClientBuilder` instance for custom configuration.
    #[inline]
    pub fn builder() -> crate::builder::ClientBuilder {
        Default::default()
    }

    /// Creates a new `Client` with the specified base URL and credentials.
    ///
    /// # Errors
    ///
    /// Returns a `reqwest::Error` if the inner HTTP client fails to build.
    pub fn new(
        base_url: impl Into<Cow<'static, str>>,
        credentials: Credentials,
    ) -> reqwest::Result<Self> {
        Ok(Self {
            base_url: base_url.into(),
            credentials,
            inner: reqwest::ClientBuilder::new()
                .user_agent(USER_AGENT)
                .build()?,
        })
    }
}

/// A type alias for results returned by pCloud API operations.
pub type Result<V> = std::result::Result<V, Error>;

/// Errors that can occur when using the pCloud client or interacting with the API.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// An error response from the API, including status code and message.
    #[error("protocol error status {0}: {1}")]
    Protocol(u16, String),
    /// A network-related error from the underlying HTTP client.
    #[error("network error")]
    Reqwest(
        #[from]
        #[source]
        reqwest::Error,
    ),
    /// An error occurred while parsing a JSON response.
    #[error("unable to decode pcloud response")]
    SerdeJson(
        #[from]
        #[source]
        serde_json::Error,
    ),
    /// An I/O error occurred while downloading a file.
    #[error("unable to download file")]
    Download(#[source] std::io::Error),
    /// An I/O error occurred while uploading a file.
    #[error("unable to upload file")]
    Upload(#[source] std::io::Error),
}
