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

    pub fn with_get_auth(mut self, value: bool) -> Self {
        self.get_auth = value;
        self
    }

    pub fn set_logout(&mut self, value: bool) {
        self.logout = value;
    }

    pub fn with_logout(mut self, value: bool) -> Self {
        self.logout = value;
        self
    }
}
#[cfg(feature = "client-http")]
mod http {
    use super::{UserInfo, UserInfoCommand};
    use crate::client::HttpClient;
    use crate::error::Error;
    use crate::prelude::HttpCommand;

    #[derive(serde::Serialize)]
    struct UserInfoParams {
        #[serde(
            rename = "getauth",
            skip_serializing_if = "crate::client::is_false",
            serialize_with = "crate::client::serialize_bool"
        )]
        get_auth: bool,
        #[serde(
            rename = "getauth",
            skip_serializing_if = "crate::client::is_false",
            serialize_with = "crate::client::serialize_bool"
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
            client.get_request::<UserInfo, _>("userinfo", params).await
        }
    }
}

#[cfg(all(test, feature = "protected", feature = "client-http"))]
mod http_tests {
    use super::UserInfoCommand;
    use crate::client::HttpClientBuilder;
    use crate::prelude::HttpCommand;

    #[tokio::test]
    async fn success() {
        let client = HttpClientBuilder::from_env().build().unwrap();
        UserInfoCommand::default().execute(&client).await.unwrap();
    }
}
