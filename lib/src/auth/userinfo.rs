use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::error::Error;
use crate::http::HttpClient;
use crate::prelude::Command;
use crate::request::Response;

#[derive(serde::Deserialize)]
pub struct UserInfo {
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
pub struct UserInfoCommand {
    get_auth: bool,
    logout: bool,
}

impl UserInfoCommand {
    pub fn new(get_auth: bool, logout: bool) -> Self {
        Self { get_auth, logout }
    }
}

impl UserInfoCommand {
    fn to_http_params(&self) -> Vec<(&str, String)> {
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

#[async_trait::async_trait(?Send)]
impl Command for UserInfoCommand {
    type Output = UserInfo;
    type Error = Error;

    async fn execute(mut self, client: &HttpClient) -> Result<Self::Output, Self::Error> {
        let result: Response<UserInfo> = client
            .get_request("userinfo", &self.to_http_params())
            .await?;
        result.payload()
    }
}

impl BinaryClient {
    #[tracing::instrument(skip(self))]
    pub fn user_info(&mut self, params: &UserInfoCommand) -> Result<UserInfo, Error> {
        let result = self.send_command("userinfo", &params.to_binary_params())?;
        let result: Response<UserInfo> = serde_json::from_value(result)?;
        result.payload()
    }
}

#[cfg(all(test, feature = "protected"))]
mod tests {
    use super::UserInfoCommand;
    use crate::binary::BinaryClient;
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
    use crate::prelude::Command;
    use crate::region::Region;

    #[tokio::test]
    async fn http_success() {
        let creds = Credentials::from_env();
        let client = HttpClient::new_eu(creds);
        UserInfoCommand::default().execute(&client).await.unwrap();
    }

    #[test]
    fn binary_success() {
        let creds = Credentials::from_env();
        let mut client = BinaryClient::new(creds, Region::eu()).unwrap();
        let params = super::Params::default();
        client.user_info(&params).unwrap();
    }
}
