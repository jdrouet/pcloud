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
#[cfg(feature = "client-http")]
mod http {
    use super::{UserInfo, UserInfoCommand};
    use crate::error::Error;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::request::Response;

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
    }

    #[async_trait::async_trait]
    impl HttpCommand for UserInfoCommand {
        type Output = UserInfo;

        async fn execute(mut self, client: &HttpClient) -> Result<Self::Output, Error> {
            let result: Response<UserInfo> = client
                .get_request("userinfo", &self.to_http_params())
                .await?;
            result.payload()
        }
    }
}

#[cfg(all(test, feature = "protected", feature = "client-http"))]
mod http_tests {
    use super::UserInfoCommand;
    use crate::http::HttpClientBuilder;
    use crate::prelude::HttpCommand;

    #[tokio::test]
    async fn success() {
        let client = HttpClientBuilder::from_env().build().unwrap();
        UserInfoCommand::default().execute(&client).await.unwrap();
    }
}
