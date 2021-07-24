mod common;
pub mod credentials;
pub mod data_center;
pub mod file;
pub mod folder;
pub mod request;

pub const DEFAULT_PART_SIZE: usize = 10485760;

/// Client for the pCloud REST API
#[derive(Clone, Debug)]
pub struct PCloudApi {
    client: reqwest::Client,
    credentials: credentials::Credentials,
    data_center: data_center::DataCenter,
    upload_part_size: usize,
}

impl PCloudApi {
    /// Create new client for the pCloud REST API
    ///
    /// # Arguments
    ///
    /// * `credentials` - Credentials to use to connect.
    /// * `data_center` - Data Center (or Region) to connect to.
    ///
    /// # Returns
    ///
    /// A new instance of the client
    pub fn new(
        credentials: credentials::Credentials,
        data_center: data_center::DataCenter,
    ) -> Self {
        Self {
            client: Self::create_client(),
            credentials,
            data_center,
            upload_part_size: DEFAULT_PART_SIZE,
        }
    }

    pub fn new_eu(credentials: credentials::Credentials) -> Self {
        Self::new(credentials, data_center::DataCenter::Europe)
    }

    pub fn new_us(credentials: credentials::Credentials) -> Self {
        Self::new(credentials, data_center::DataCenter::UnitedStates)
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
