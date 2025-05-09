#[derive(Debug, serde::Deserialize)]
pub struct Token {
    #[serde(rename = "auth")]
    pub value: String,
}

#[derive(serde::Serialize)]
struct Params {
    getauth: u8,
}

impl Default for Params {
    fn default() -> Self {
        Self { getauth: 1 }
    }
}

impl crate::Client {
    /// Generate a new token based on the provided credentials
    ///
    /// # Returns
    ///
    /// A [`Digest`] struct containing the digest itself
    ///
    /// # Errors
    ///
    /// Returns a [`crate::Error`] if the folder cannot be listed.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example(client: &pcloud::Client) -> Result<(), pcloud::Error> {
    /// let value = client.get_digest().await?;
    /// println!("{:?}", value);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_token(&self) -> crate::Result<String> {
        self.get_request::<Token, _>("userinfo", &Params::default())
            .await
            .map(|token| token.value)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Client, Credentials};
    use mockito::Matcher;

    #[tokio::test]
    async fn success() {
        let mut server = mockito::Server::new_async().await;
        let m = server.mock("GET", "/getdigest")
            .match_query(Matcher::Missing)
            .with_status(200)
            .with_body(r#"{"result": 0, "digest": "YGtAxbUpI85Zvs7lC7Z62rBwv907TBXhV2L867Hkh", "expires": "Fri, 27 Sep 2013 10:15:46 +0000"}"#)
            .create_async().await;
        let client = Client::new(server.url(), Credentials::anonymous()).unwrap();
        let result = client.get_digest().await.unwrap();
        assert_eq!(result.value, "YGtAxbUpI85Zvs7lC7Z62rBwv907TBXhV2L867Hkh");
        m.assert_async().await;
    }
}
