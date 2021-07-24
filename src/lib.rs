mod common;
pub mod credentials;
pub mod error;
pub mod file;
pub mod folder;
pub mod region;
pub mod request;

pub const DEFAULT_PART_SIZE: usize = 10485760;

/// Client for the pCloud REST API
#[derive(Clone, Debug)]
pub struct PCloudApi {
    client: reqwest::Client,
    credentials: credentials::Credentials,
    region: region::Region,
    upload_part_size: usize,
}

impl PCloudApi {
    /// Create new client for the pCloud REST API
    ///
    /// # Arguments
    ///
    /// * `credentials` - Credentials to use to connect.
    /// * `region` - Region to connect to.
    ///
    /// # Returns
    ///
    /// A new instance of the client
    pub fn new(credentials: credentials::Credentials, region: region::Region) -> Self {
        Self {
            client: Self::create_client(),
            credentials,
            region,
            upload_part_size: DEFAULT_PART_SIZE,
        }
    }

    pub fn new_eu(credentials: credentials::Credentials) -> Self {
        Self::new(credentials, region::Region::Europe)
    }

    pub fn new_us(credentials: credentials::Credentials) -> Self {
        Self::new(credentials, region::Region::UnitedStates)
    }
}

impl PCloudApi {
    pub fn upload_part_size(mut self, value: usize) -> Self {
        self.upload_part_size = value;
        self
    }
}

#[cfg(test)]
mod tests {
    pub fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }
}
