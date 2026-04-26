/// Response payload returned by the pCloud `oauth2_token` endpoint.
#[derive(Debug, serde::Deserialize)]
pub struct OAuth2Token {
    pub access_token: String,
    #[serde(default)]
    pub token_type: Option<String>,
    #[serde(default)]
    pub uid: Option<u64>,
    #[serde(default)]
    pub locationid: Option<u64>,
}

#[derive(serde::Serialize)]
struct Params<'a> {
    client_id: &'a str,
    client_secret: &'a str,
    code: &'a str,
}

impl crate::Client {
    /// Exchanges an OAuth2 authorization `code` for an access token.
    ///
    /// This method does not use the client's configured credentials; the OAuth2
    /// flow authenticates via the `client_id`, `client_secret` and `code`
    /// parameters directly.
    ///
    /// See <https://docs.pcloud.com/methods/oauth_2.0/authorize.html>.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::Error`] if the request fails or the server rejects
    /// the exchange.
    pub async fn oauth2_token(
        &self,
        client_id: &str,
        client_secret: &str,
        code: &str,
    ) -> crate::Result<OAuth2Token> {
        self.get_request_unauthenticated(
            "oauth2_token",
            &Params {
                client_id,
                client_secret,
                code,
            },
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use crate::{Client, Credentials};
    use mockito::Matcher;

    #[tokio::test]
    async fn success() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("GET", "/oauth2_token")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("client_id".into(), "id".into()),
                Matcher::UrlEncoded("client_secret".into(), "secret".into()),
                Matcher::UrlEncoded("code".into(), "abc".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{"result": 0, "access_token": "the-token", "token_type": "bearer", "uid": 42, "locationid": 1}"#,
            )
            .create_async()
            .await;
        let client = Client::new(server.url(), Credentials::anonymous()).unwrap();
        let result = client.oauth2_token("id", "secret", "abc").await.unwrap();
        assert_eq!(result.access_token, "the-token");
        assert_eq!(result.token_type.as_deref(), Some("bearer"));
        assert_eq!(result.uid, Some(42));
        assert_eq!(result.locationid, Some(1));
        m.assert_async().await;
    }

    #[tokio::test]
    async fn protocol_error() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("GET", "/oauth2_token")
            .match_query(Matcher::Any)
            .with_status(200)
            .with_body(r#"{"result": 2000, "error": "invalid code"}"#)
            .create_async()
            .await;
        let client = Client::new(server.url(), Credentials::anonymous()).unwrap();
        let err = client
            .oauth2_token("id", "secret", "bad")
            .await
            .unwrap_err();
        assert!(matches!(err, crate::Error::Protocol(2000, _)));
        m.assert_async().await;
    }
}
