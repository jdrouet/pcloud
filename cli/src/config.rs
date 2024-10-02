use pcloud::client::{HttpClient, HttpClientBuilder, HttpClientBuilderError};
use pcloud::credentials::Credentials;
use pcloud::region::Region;
use serde::Deserialize;
use std::path::Path;
use std::time::Duration;

#[derive(Deserialize)]
pub struct CredentialsConfig {
    username: String,
    password: String,
}

impl CredentialsConfig {
    fn build(self) -> Credentials {
        Credentials::UserPassword {
            username: self.username,
            password: self.password,
        }
    }
}

#[derive(Deserialize)]
pub struct RegionConfig {
    name: String,
}

impl RegionConfig {
    fn build(self) -> Region {
        Region::from_name(self.name.as_str()).unwrap_or_default()
    }
}

#[derive(Default, Deserialize)]
pub struct Config {
    credentials: Option<CredentialsConfig>,
    region: Option<RegionConfig>,
    timeout: Option<u64>,
}

impl Config {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let reader = std::fs::File::open(path).map_err(|err| err.to_string())?;
        let result = serde_json::from_reader(reader).map_err(|err| err.to_string())?;
        Ok(result)
    }

    pub fn build(self) -> Result<HttpClient, HttpClientBuilderError> {
        let mut builder = HttpClientBuilder::from_env();
        if let Some(timeout) = self.timeout.map(Duration::from_secs) {
            builder.timeout = Some(timeout);
        }
        if let Some(creds) = self.credentials.map(|c| c.build()) {
            builder.credentials = Some(creds);
        }
        if let Some(region) = self.region.map(|c| c.build()) {
            builder.region = Some(region);
        }
        builder.build()
    }
}
