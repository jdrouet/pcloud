use pcloud::builder::Error as ClientBuilderError;
use pcloud::Client;
use pcloud::Credentials;
use pcloud::Region;
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
        Credentials::username_password(self.username, self.password)
    }
}

#[derive(Default, Deserialize)]
pub struct Config {
    credentials: Option<CredentialsConfig>,
    region: Option<Region>,
    timeout: Option<u64>,
}

impl Config {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let reader = std::fs::File::open(path).map_err(|err| err.to_string())?;
        let result = serde_json::from_reader(reader).map_err(|err| err.to_string())?;
        Ok(result)
    }

    pub fn build(self) -> Result<Client, ClientBuilderError> {
        let mut reqwest_builder = pcloud::reqwest::Client::builder();

        let mut builder = Client::builder();
        if let Some(timeout) = self.timeout.map(Duration::from_secs) {
            reqwest_builder = reqwest_builder.timeout(timeout);
        }
        if let Some(creds) = self.credentials.map(|c| c.build()) {
            builder.set_credentials(creds);
        }
        if let Some(region) = self.region {
            builder.set_region(region);
        }
        builder.set_client_builder(reqwest_builder);
        builder.build()
    }
}
