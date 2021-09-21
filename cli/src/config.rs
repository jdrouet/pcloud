use pcloud::credentials::Credentials;
use pcloud::http::HttpClient;
use pcloud::region::Region;
use serde::Deserialize;
use std::path::Path;

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

#[derive(Deserialize)]
pub struct Config {
    credentials: CredentialsConfig,
    region: RegionConfig,
}

impl Config {
    pub fn from_path(path: &Path) -> Result<Self, String> {
        let reader = std::fs::File::open(path).map_err(|err| err.to_string())?;
        let result = serde_json::from_reader(reader).map_err(|err| err.to_string())?;
        Ok(result)
    }

    pub fn build(self) -> HttpClient {
        let creds = self.credentials.build();
        let region = self.region.build();
        HttpClient::new(creds, region)
    }
}
