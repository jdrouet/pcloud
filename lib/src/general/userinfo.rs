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

    #[derive(serde::Serialize)]
    struct UserInfoParams {
        #[serde(
            rename = "getauth",
            skip_serializing_if = "crate::http::is_false",
            serialize_with = "crate::http::serialize_bool"
        )]
        get_auth: bool,
        #[serde(
            rename = "getauth",
            skip_serializing_if = "crate::http::is_false",
            serialize_with = "crate::http::serialize_bool"
        )]
        logout: bool,
    }

    impl From<UserInfoCommand> for UserInfoParams {
        fn from(value: UserInfoCommand) -> Self {
            Self {
                get_auth: value.get_auth,
                logout: value.logout,
            }
        }
    }

    #[async_trait::async_trait]
    impl HttpCommand for UserInfoCommand {
        type Output = UserInfo;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let params = UserInfoParams::from(self);
            let result: Response<UserInfo> = client.get_request("userinfo", &params).await?;
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
