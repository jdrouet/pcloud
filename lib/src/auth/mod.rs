use crate::error::Error;
use crate::http::PCloudApi;
use crate::request::Response;

#[derive(serde::Deserialize)]
struct Payload {
    auth: String,
}

impl PCloudApi {
    pub async fn get_token(&self) -> Result<String, Error> {
        let params = vec![("getauth", "1"), ("logout", "1")];
        let result: Response<Payload> = self.get_request("userinfo", &params).await?;
        result.payload().map(|item| item.auth)
    }
}
