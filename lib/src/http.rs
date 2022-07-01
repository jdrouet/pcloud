//! The client implementing the [HTTP Json protocol](https://docs.pcloud.com/protocols/http_json_protocol/)

use crate::credentials::Credentials;
use crate::error::Error;
use crate::region::Region;
use std::time::Duration;

/// The default user agent for the http client
pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
/// The default part size when uploading files
pub const DEFAULT_PART_SIZE: usize = 10485760;

/// The errors when generating a [`HttpClient`](HttpClient) from a [`HttpClientBuilder`](HttpClientBuilder)
#[derive(Debug)]
pub enum HttpClientBuilderError {
    CredentialsMissing,
    Reqwest(reqwest::Error),
}

/// A builder for the [`HttpClient`](HttpClient) structure
///
/// ```
/// use pcloud::http::HttpClientBuilder;
/// use pcloud::credentials::Credentials;
/// use pcloud::region::Region;
///
/// let _client = HttpClientBuilder::default()
///    .credentials(Credentials::AccessToken("my-token".to_string()))
///    .region(Region::eu())
///    .build()
///    .expect("unable to builder http client");
/// ```
#[derive(Debug, Default)]
pub struct HttpClientBuilder {
    pub client_builder: reqwest::ClientBuilder,
    pub credentials: Option<Credentials>,
    pub region: Option<Region>,
    pub timeout: Option<Duration>,
}

// TODO handle the parsing error gracefully
fn duration_from_env() -> Option<Duration> {
    std::env::var("PCLOUD_TIMEOUT")
        .ok()
        .map(|value| {
            value
                .parse::<u64>()
                .expect("invalid value for PCLOUD_TIMEOUT environment variable")
        })
        .map(Duration::from_millis)
}

impl HttpClientBuilder {
    /// Builds a http client builder from the environment variables. See [`Credentials`](crate::credentials::Credentials) and [`Region`](crate::region::Region).
    ///
    /// The timeout value will be the value from the `PCLOUD_TIMEOUT` environment variable, in milliseconds.
    /// If the value is not a valid number, the function will panic.
    pub fn from_env() -> Self {
        Self {
            client_builder: reqwest::ClientBuilder::default(),
            credentials: Credentials::from_env(),
            region: Region::from_env(),
            timeout: duration_from_env(),
        }
    }

    pub fn client_builder(mut self, value: reqwest::ClientBuilder) -> Self {
        self.client_builder = value;
        self
    }

    pub fn credentials(mut self, value: Credentials) -> Self {
        self.credentials = Some(value);
        self
    }

    pub fn region(mut self, value: Region) -> Self {
        self.region = Some(value);
        self
    }

    pub fn timeout(mut self, value: Duration) -> Self {
        self.timeout = Some(value);
        self
    }

    /// Builds a client for the http protocol
    ///
    /// Returns `Ok(client)` on success, otherwise returns an error.
    ///
    /// # Errors
    ///
    /// Returns `Err(HttpClientBuilderError::CredentialsMissing)` when the credentials are not provided.
    /// Returns `Err(HttpClientBuilderError::Reqwest)` when the reqwest client cannot be built.
    ///
    /// # Example
    ///
    /// ```rust
    /// use pcloud::http::HttpClientBuilder;
    /// use pcloud::http::HttpClientBuilderError;
    ///
    /// match HttpClientBuilder::default().build() {
    ///     Ok(_client) => println!("success!"),
    ///     Err(HttpClientBuilderError::CredentialsMissing) => eprintln!("no credentials provided"),
    ///     Err(HttpClientBuilderError::Reqwest(err)) => eprintln!("unable to build reqwest client: {:?}", err),
    /// }
    /// ```
    pub fn build(self) -> Result<HttpClient, HttpClientBuilderError> {
        let client_builder = if let Some(timeout) = self.timeout {
            self.client_builder.timeout(timeout)
        } else {
            self.client_builder
        };
        Ok(HttpClient {
            client: client_builder
                .build()
                .map_err(HttpClientBuilderError::Reqwest)?,
            credentials: self
                .credentials
                .ok_or(HttpClientBuilderError::CredentialsMissing)?,
            region: self.region.unwrap_or_default(),
        })
    }
}

/// Client for the pCloud REST API
///
/// ```rust
/// use pcloud::http::HttpClientBuilder;
/// use pcloud::credentials::Credentials;
/// use pcloud::region::Region;
/// use pcloud::general::userinfo::UserInfoCommand;
/// use pcloud::prelude::HttpCommand;
///
/// # tokio_test::block_on(async {
/// let client = HttpClientBuilder::from_env()
///    .build()
///    .expect("unable to builder binary client");
/// let result = UserInfoCommand::new(false, false)
///    .execute(&client)
///    .await
///    .expect("unable to execute command");
/// # })
/// ```
#[derive(Clone)]
pub struct HttpClient {
    pub(crate) client: reqwest::Client,
    credentials: Credentials,
    region: Region,
}

#[cfg(test)]
impl HttpClient {
    pub fn new(credentials: Credentials, region: Region) -> Self {
        Self {
            client: reqwest::ClientBuilder::default()
                .user_agent(USER_AGENT)
                .build()
                .unwrap(),
            credentials,
            region,
        }
    }
}

async fn read_response<T: serde::de::DeserializeOwned>(
    action: &str,
    method: &str,
    res: reqwest::Response,
) -> Result<T, Error> {
    if cfg!(test) {
        let body = res.text().await?;
        println!("{} {}: {}", action, method, body);
        Ok(serde_json::from_str(&body).unwrap())
    } else {
        res.json::<T>().await.map_err(Error::from)
    }
}

impl HttpClient {
    fn build_url(&self, method: &str) -> String {
        format!("{}/{}", self.region.http_url(), method)
    }

    pub(crate) async fn get_request<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: &[(&str, String)],
    ) -> Result<T, Error> {
        let mut local_params = self.credentials.to_http_params();
        local_params.extend_from_slice(params);
        let uri = self.build_url(method);
        let res = self.client.get(uri).query(&local_params).send().await?;
        read_response("GET", method, res).await
    }

    pub(crate) async fn put_request_data<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: &[(&str, String)],
        payload: Vec<u8>,
    ) -> Result<T, Error> {
        let mut local_params = self.credentials.to_http_params();
        local_params.extend_from_slice(params);
        let uri = self.build_url(method);
        let res = self
            .client
            .put(uri)
            .query(&local_params)
            .body(payload)
            .send()
            .await?;
        read_response("PUT", method, res).await
    }
}
