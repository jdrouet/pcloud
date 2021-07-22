mod common;
pub mod credentials;
pub mod data_center;
pub mod folder;
pub mod request;

#[derive(Clone, Debug)]
pub struct PCloudApi {
    client: reqwest::Client,
    credentials: credentials::Credentials,
    data_center: data_center::DataCenter,
}

impl PCloudApi {
    pub fn new(
        credentials: credentials::Credentials,
        data_center: data_center::DataCenter,
    ) -> Self {
        Self {
            client: Self::create_client(),
            credentials,
            data_center,
        }
    }

    pub fn new_eu(credentials: credentials::Credentials) -> Self {
        Self::new(credentials, data_center::DataCenter::Europe)
    }

    pub fn new_us(credentials: credentials::Credentials) -> Self {
        Self::new(credentials, data_center::DataCenter::UnitedStates)
    }
}
