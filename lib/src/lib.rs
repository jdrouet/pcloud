use std::borrow::Cow;

pub mod builder;
pub mod entry;
pub mod file;
pub mod folder;
pub mod stream;

mod date;
mod request;

// re exporting dependencies
pub use reqwest;

/// The default user agent for the http client
const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));

pub const EU_REGION: &str = "https://eapi.pcloud.com";
pub const US_REGION: &str = "https://api.pcloud.com";

#[derive(Clone, Copy, Debug, serde::Serialize, serde::Deserialize)]
pub enum Region {
    #[serde(alias = "EU")]
    Eu,
    #[serde(alias = "US")]
    Us,
}

impl Region {
    const fn base_url(&self) -> &'static str {
        match self {
            Self::Eu => EU_REGION,
            Self::Us => US_REGION,
        }
    }
}

impl Region {
    pub fn from_env() -> Option<Self> {
        let name = std::env::var("PCLOUD_REGION").ok()?;
        match name.as_str() {
            "eu" | "EU" => Some(Self::Eu),
            "us" | "US" => Some(Self::Us),
            _ => None,
        }
    }
}

#[derive(serde::Serialize, serde::Deserialize)]
#[serde(untagged)]
pub enum Credentials {
    AccessToken { access_token: String },
    UsernamePassword { username: String, password: String },
}

impl Credentials {
    pub fn access_token(value: impl Into<String>) -> Self {
        Self::AccessToken {
            access_token: value.into(),
        }
    }

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

#[derive(Debug)]
pub struct Client {
    base_url: Cow<'static, str>,
    credentials: Credentials,
    inner: reqwest::Client,
}

impl Client {
    #[inline]
    pub fn builder() -> crate::builder::ClientBuilder {
        Default::default()
    }

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

pub type Result<V> = std::result::Result<V, Error>;

/// All the possible errors returned by the clients and the API
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Server side error, properly handled, returning a code and a message
    #[error("protocol error status {0}: {1}")]
    Protocol(u16, String),
    /// Error specific to the [`Client`](crate::Client)
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
