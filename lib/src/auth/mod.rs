use crate::error::Error;
use crate::http::PCloudHttpApi;
use crate::request::Response;

#[derive(serde::Deserialize)]
struct Payload {
    auth: String,
}

impl PCloudHttpApi {
    pub async fn get_token(&self) -> Result<String, Error> {
        let params = vec![("getauth", "1".to_string()), ("logout", "1".to_string())];
        let result: Response<Payload> = self.get_request("userinfo", &params).await?;
        result.payload().map(|item| item.auth)
    }
}
