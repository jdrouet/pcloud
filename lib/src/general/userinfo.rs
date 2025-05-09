use chrono::{DateTime, Utc};

#[derive(Debug, serde::Serialize, serde::Deserialize)]
pub struct UserInfo {
    pub email: String,
    #[serde(rename = "emailverified")]
    pub email_verified: bool,
    #[serde(with = "crate::date")]
    pub registered: DateTime<Utc>,
    #[serde(default)]
    pub premium: bool,
    #[serde(default, rename = "premiumexpires", with = "crate::date::optional")]
    pub premium_expires: Option<DateTime<Utc>>,
    pub quota: u64,
    #[serde(rename = "usedquota")]
    pub used_quota: u64,
    pub language: String,
}

impl crate::Client {
    /// Fetches the information about the current user.
    ///
    /// # Returns
    ///
    /// A [`UserInfo`] struct containing the user information.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::Error`] if the folder cannot be listed.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example(client: &pcloud::Client) -> Result<(), pcloud::Error> {
    /// let info = client.user_info().await?;
    /// println!("{:?}", info);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn user_info(&self) -> crate::Result<UserInfo> {
        self.get_request("userinfo", &()).await
    }
}
