use std::borrow::Cow;

/// Errors that may occur during client configuration and building.
#[derive(Debug, thiserror::Error)]
pub enum Error {
    /// Returned when the underlying HTTP client could not be built.
    #[error("unable to build reqwest client")]
    Reqwest(#[from] reqwest::Error),
}

/// Builder for constructing a [`Client`](crate::Client) with custom configuration.
///
/// This allows specifying the API region, base URL, credentials, and optionally
/// customizing the inner `reqwest::ClientBuilder`.
#[derive(Debug)]
pub struct ClientBuilder {
    base_url: Cow<'static, str>,
    client_builder: Option<reqwest::ClientBuilder>,
    credentials: crate::Credentials,
}

impl Default for ClientBuilder {
    /// Creates a new `ClientBuilder` with default settings:
    ///
    /// - Base URL is set to the EU region.
    /// - No credentials are set.
    /// - No custom `reqwest::ClientBuilder` is used.
    fn default() -> Self {
        Self {
            base_url: Cow::Borrowed(crate::EU_REGION),
            client_builder: None,
            credentials: crate::Credentials::Anonymous,
        }
    }
}

impl ClientBuilder {
    /// Creates a builder pre-configured using environment variables.
    ///
    /// - Uses `PCLOUD_REGION` or `PCLOUD_BASE_URL` for the endpoint.
    /// - Uses `PCLOUD_ACCESS_TOKEN` or `PCLOUD_USERNAME`/`PCLOUD_PASSWORD` for credentials.
    ///
    /// Falls back to the EU region if none is specified.
    pub fn from_env() -> Self {
        let base_url = crate::Region::from_env()
            .map(|region| Cow::Borrowed(region.base_url()))
            .or_else(|| std::env::var("PCLOUD_BASE_URL").ok().map(Cow::Owned))
            .unwrap_or(Cow::Borrowed(crate::EU_REGION));
        let credentials = crate::Credentials::from_env().unwrap_or(crate::Credentials::Anonymous);

        Self {
            base_url,
            client_builder: None,
            credentials,
        }
    }
}

impl ClientBuilder {
    /// Sets the API region.
    pub fn set_region(&mut self, region: crate::Region) {
        self.base_url = region.base_url().into();
    }

    /// Sets the API region and returns the modified builder.
    pub fn with_region(mut self, region: crate::Region) -> Self {
        self.set_region(region);
        self
    }

    /// Sets a custom base URL.
    pub fn set_base_url(&mut self, base_url: impl Into<Cow<'static, str>>) {
        self.base_url = base_url.into();
    }

    /// Sets a custom base URL and returns the modified builder.
    pub fn with_base_url(mut self, base_url: impl Into<Cow<'static, str>>) -> Self {
        self.set_base_url(base_url);
        self
    }

    /// Sets a custom `reqwest::ClientBuilder`.
    pub fn set_client_builder(&mut self, client_builder: reqwest::ClientBuilder) {
        self.client_builder = Some(client_builder);
    }

    /// Sets a custom `reqwest::ClientBuilder` and returns the modified builder.
    pub fn with_client_builder(mut self, client_builder: reqwest::ClientBuilder) -> Self {
        self.set_client_builder(client_builder);
        self
    }

    /// Sets the credentials for API authentication.
    pub fn set_credentials(&mut self, credentials: crate::Credentials) {
        self.credentials = credentials;
    }

    /// Sets the credentials and returns the modified builder.
    pub fn with_credentials(mut self, credentials: crate::Credentials) -> Self {
        self.set_credentials(credentials);
        self
    }

    /// Builds the [`Client`](crate::Client) with the configured options.
    ///
    /// # Errors
    ///
    /// Returns [`Error::MissingCredentials`] if no credentials were set.
    /// Returns [`Error::Reqwest`] if the HTTP client could not be built.
    pub fn build(self) -> Result<crate::Client, Error> {
        let builder = self
            .client_builder
            .unwrap_or_default()
            .user_agent(crate::USER_AGENT);
        Ok(crate::Client {
            base_url: self.base_url,
            credentials: self.credentials,
            inner: builder.build()?,
        })
    }
}
