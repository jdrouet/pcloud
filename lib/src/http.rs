use crate::credentials::Credentials;
use crate::error::Error;
use crate::region::Region;

pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
pub const DEFAULT_PART_SIZE: usize = 10485760;

/// Client for the pCloud REST API
#[derive(Clone)]
pub struct HttpClient {
    pub(crate) client: reqwest::Client,
    credentials: Credentials,
    region: Region,
    pub(crate) upload_part_size: usize,
}

impl HttpClient {
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
    pub fn new(credentials: Credentials, region: Region) -> Self {
        Self {
            client: Self::create_client(),
            credentials,
            region,
            upload_part_size: DEFAULT_PART_SIZE,
        }
    }

    pub fn new_eu(credentials: Credentials) -> Self {
        Self::new(credentials, Region::eu())
    }

    pub fn new_us(credentials: Credentials) -> Self {
        Self::new(credentials, Region::us())
    }

    pub fn from_env() -> Self {
        Self::new(Credentials::from_env(), Region::from_env())
    }
}

impl HttpClient {
    pub fn upload_part_size(mut self, value: usize) -> Self {
        self.upload_part_size = value;
        self
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
    pub(crate) fn create_client() -> reqwest::Client {
        reqwest::ClientBuilder::new()
            .user_agent(USER_AGENT)
            .build()
            .expect("couldn't create reqwest client")
    }

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
