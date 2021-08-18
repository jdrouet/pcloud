#[derive(Clone, Debug)]
pub enum Credentials {
    AccessToken(String),
    UserPassword { username: String, password: String },
}

impl Credentials {
    pub fn to_vec(&self) -> Vec<(&str, String)> {
        match self {
            Self::AccessToken(value) => vec![("access_token", value.clone())],
            Self::UserPassword { username, password } => vec![
                ("username", username.clone()),
                ("password", password.clone()),
            ],
        }
    }
}

impl Credentials {
    pub fn from_env() -> Self {
        if let Ok(access_token) = std::env::var("PCLOUD_ACCESS_TOKEN") {
            Self::AccessToken(access_token)
        } else if let (Ok(username), Ok(password)) = (
            std::env::var("PCLOUD_USERNAME"),
            std::env::var("PCLOUD_PASSWORD"),
        ) {
            Self::UserPassword { username, password }
        } else {
            panic!("unable to build from environment");
        }
    }
}
