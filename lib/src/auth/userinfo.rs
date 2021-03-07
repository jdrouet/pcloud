use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::error::Error;
use crate::http::HttpClient;
use crate::request::Response;

#[derive(serde::Deserialize)]
pub struct Payload {
    pub email: String,
    #[serde(rename = "emailverified")]
    pub email_verified: bool,
    pub premium: bool,
    pub quota: usize,
    #[serde(rename = "usedquota")]
    pub used_quota: usize,
    pub language: String,
    pub auth: Option<String>,
}

#[derive(Debug, Default)]
pub struct Params {
    get_auth: bool,
    logout: bool,
}

impl Params {
    pub fn new(get_auth: bool, logout: bool) -> Self {
        Self { get_auth, logout }
    }

    pub fn to_http_params(&self) -> Vec<(&str, String)> {
        let mut res = Vec::new();
        if self.get_auth {
            res.push(("getauth", 1.to_string()));
        }
        if self.logout {
            res.push(("logout", 1.to_string()));
        }
        res
    }

    pub fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        let mut res = Vec::new();
        if self.get_auth {
            res.push(("getauth", BinaryValue::Bool(true)));
        }
        if self.logout {
            res.push(("logout", BinaryValue::Bool(true)));
        }
        res
    }
}

impl HttpClient {
    #[tracing::instrument(skip(self))]
    pub async fn user_info(&self, params: &Params) -> Result<Payload, Error> {
        let result: Response<Payload> = self
            .get_request("userinfo", &params.to_http_params())
            .await?;
        result.payload()
    }
}

impl BinaryClient {
    #[tracing::instrument(skip(self))]
    pub fn user_info(&mut self, params: &Params) -> Result<Payload, Error> {
        let result = self.send_command("userinfo", &params.to_binary_params())?;
        let result: Response<Payload> = serde_json::from_value(result)?;
        result.payload()
    }
}

#[cfg(all(test, feature = "protected"))]
mod tests {
    use crate::binary::BinaryClient;
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
    use crate::region::Region;

    #[tokio::test]
    async fn http_success() {
        let creds = Credentials::from_env();
        let client = HttpClient::new_eu(creds);
        let params = super::Params::default();
        client.user_info(&params).await.unwrap();
    }

    #[test]
    fn binary_success() {
        let creds = Credentials::from_env();
        let mut client = BinaryClient::new(creds, Region::eu()).unwrap();
        let params = super::Params::default();
        client.user_info(&params).unwrap();
    }
}
