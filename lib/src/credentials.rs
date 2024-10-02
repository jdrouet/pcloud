//! This gives the required structure to authenticate with the PCloud API as specified in [the documentation](https://docs.pcloud.com/methods/intro/authentication.html).

/// The different kind of credentials used for authentication
#[derive(Clone, Debug)]
#[cfg_attr(feature = "client-http", derive(serde::Serialize))]
#[cfg_attr(feature = "client-http", serde(untagged))]
pub enum Credentials {
    AccessToken { access_token: String },
    UserPassword { username: String, password: String },
}

impl Credentials {
    /// Creates a credential based on the environment variables
    ///
    /// When `PCLOUD_ACCESS_TOKEN` is set, a `Some(Credentials::AccessToken)` will be created.
    ///
    /// When `PCLOUD_USERNAME` and `PCLOUD_PASSWORD` are set, a `Some(Credentials::UserPassword)` will be created.
    ///
    /// If none are set, `None` is returned.
    ///
    /// ```rust
    /// use pcloud::credentials::Credentials;
    ///
    /// match Credentials::from_env() {
    ///     Some(Credentials::AccessToken { .. }) => println!("uses an access token"),
    ///     Some(Credentials::UserPassword { .. }) => println!("uses a username and a password"),
    ///     None => eprintln!("no credentials provided"),
    /// }
    /// ```
    pub fn from_env() -> Option<Self> {
        if let Ok(access_token) = std::env::var("PCLOUD_ACCESS_TOKEN") {
            Some(Self::AccessToken { access_token })
        } else if let (Ok(username), Ok(password)) = (
            std::env::var("PCLOUD_USERNAME"),
            std::env::var("PCLOUD_PASSWORD"),
        ) {
            Some(Self::UserPassword { username, password })
        } else {
            None
        }
    }
}

impl Credentials {
    pub fn access_token<S: Into<String>>(access_token: S) -> Self {
        Self::AccessToken {
            access_token: access_token.into(),
        }
    }

    pub fn user_password<U: Into<String>, P: Into<String>>(username: U, password: P) -> Self {
        Self::UserPassword {
            username: username.into(),
            password: password.into(),
        }
    }
}
