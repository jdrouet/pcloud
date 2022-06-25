use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::error::Error;
use crate::http::HttpClient;
use crate::prelude::{BinaryCommand, HttpCommand};
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

    pub fn set_get_auth(&mut self, value: bool) {
        self.get_auth = value;
    }

    pub fn set_logout(&mut self, value: bool) {
        self.logout = value;
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

    fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
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
impl HttpCommand for UserInfoCommand {
    type Output = UserInfo;

    async fn execute(mut self, client: &HttpClient) -> Result<Self::Output, Error> {
        let result: Response<UserInfo> = client
            .get_request("userinfo", &self.to_http_params())
            .await?;
        result.payload()
    }
}

impl BinaryCommand for UserInfoCommand {
    type Output = UserInfo;

    fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
        let result = client.send_command("userinfo", &self.to_binary_params())?;
        let result: Response<UserInfo> = serde_json::from_value(result)?;
        result.payload()
    }
}

#[cfg(all(test, feature = "protected"))]
mod http_tests {
    use super::UserInfoCommand;
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::region::Region;

    #[tokio::test]
    async fn success() {
        let creds = Credentials::from_env().unwrap();
        let client = HttpClient::new(creds, Region::eu());
        UserInfoCommand::default().execute(&client).await.unwrap();
    }
}

#[cfg(all(test, feature = "protected"))]
mod binary_tests {
    use super::UserInfoCommand;
    use crate::binary::BinaryClient;
    use crate::credentials::Credentials;
    use crate::prelude::BinaryCommand;
    use crate::region::Region;

    #[test]
    fn success() {
        let creds = Credentials::from_env().unwrap();
        let mut client = BinaryClient::new(creds, Region::eu()).unwrap();
        let _params = UserInfoCommand::default().execute(&mut client).unwrap();
    }
}
