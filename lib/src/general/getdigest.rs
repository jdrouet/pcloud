use chrono::{DateTime, Utc};

#[derive(Debug, serde::Deserialize)]
pub struct Digest {
    #[serde(rename = "digest")]
    pub value: String,
    #[serde(with = "crate::date")]
    pub expires: DateTime<Utc>,
}

impl crate::Client {
    /// Generates an authentication digest, valid for 30 seconds.
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
    pub async fn get_digest(&self) -> crate::Result<Digest> {
        self.get_request("getdigest", &()).await
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
