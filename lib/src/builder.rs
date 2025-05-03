use std::borrow::Cow;

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("credentials not specified")]
    MissingCredentials,
    #[error("unable to build reqwest client")]
    Reqwest(#[from] reqwest::Error),
}

#[derive(Debug)]
pub struct ClientBuilder {
    base_url: Cow<'static, str>,
    client_builder: Option<reqwest::ClientBuilder>,
    credentials: Option<crate::Credentials>,
}

impl Default for ClientBuilder {
    fn default() -> Self {
        Self {
            base_url: Cow::Borrowed(crate::EU_REGION),
            client_builder: None,
            credentials: None,
        }
    }
}

impl ClientBuilder {
    pub fn set_region(&mut self, region: crate::Region) {
        self.base_url = region.base_url().into();
    }

    pub fn with_region(mut self, region: crate::Region) -> Self {
        self.set_region(region);
        self
    }

    pub fn set_base_url(&mut self, base_url: impl Into<Cow<'static, str>>) {
        self.base_url = base_url.into();
    }

    pub fn with_base_url(mut self, base_url: impl Into<Cow<'static, str>>) -> Self {
        self.set_base_url(base_url);
        self
    }

    pub fn set_client_builder(&mut self, client_builder: reqwest::ClientBuilder) {
        self.client_builder = Some(client_builder);
    }

    pub fn with_client_builder(mut self, client_builder: reqwest::ClientBuilder) -> Self {
        self.set_client_builder(client_builder);
        self
    }

    pub fn set_credentials(&mut self, credentials: crate::Credentials) {
        self.credentials = Some(credentials);
    }

    pub fn with_credentials(mut self, credentials: crate::Credentials) -> Self {
        self.set_credentials(credentials);
        self
    }

    pub fn build(self) -> Result<crate::Client, Error> {
        let credentials = self.credentials.ok_or(Error::MissingCredentials)?;
        let builder = self
            .client_builder
            .unwrap_or_default()
            .user_agent(crate::USER_AGENT);
        Ok(crate::Client {
            base_url: self.base_url,
            credentials,
            inner: builder.build()?,
        })
    }
}
