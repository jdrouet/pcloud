use super::PCloudApi;

const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"),);
pub const ROOT_FOLDER: usize = 0;

#[derive(Debug)]
pub enum Error {
    Reqwest(reqwest::Error),
    Payload(u16),
}

impl PCloudApi {
    pub(crate) fn create_client() -> reqwest::Client {
        reqwest::ClientBuilder::new()
            .user_agent(USER_AGENT)
            .build()
            .expect("couldn't create reqwest client")
    }

    fn build_url(&self, method: &str) -> String {
        format!("{}/{}", self.data_center.base_url(), method)
    }

    pub(crate) async fn get_request<T: serde::de::DeserializeOwned>(
        &self,
        method: &str,
        params: &[(&str, &str)],
    ) -> Result<T, Error> {
        let mut local_params = self.credentials.to_vec();
        local_params.extend_from_slice(params);
        let uri = self.build_url(method);
        self.client
            .get(uri)
            .query(&local_params)
            .send()
            .await
            .map_err(Error::Reqwest)?
            .json::<T>()
            .await
            .map_err(|err| {
                println!("parser error: {:?}", err);
                Error::Reqwest(err)
            })
    }
}
