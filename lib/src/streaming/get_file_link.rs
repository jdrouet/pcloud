//! Resources needed to get a link to a file in order to stream it

use crate::file::FileIdentifier;

/// Command to a file streaming link
///
/// Executing this command with return the url to the file.
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/streaming/getfilelink.html)
///
/// # Example using the [`HttpClient`](crate::http::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::http::HttpClientBuilder;
/// use pcloud::prelude::HttpCommand;
/// use pcloud::streaming::get_file_link::GetFileLinkCommand;
///
/// # tokio_test::block_on(async {
/// let client = HttpClientBuilder::from_env().build().unwrap();
/// let cmd = GetFileLinkCommand::new("/foo/bar.txt".into());
/// match cmd.execute(&client).await {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// # })
/// ```
#[derive(Debug)]
pub struct GetFileLinkCommand {
    pub identifier: FileIdentifier,
}

impl GetFileLinkCommand {
    pub fn new(identifier: FileIdentifier) -> Self {
        Self { identifier }
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::GetFileLinkCommand;
    use crate::error::Error;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::request::Response;
    use crate::streaming::Payload;

    #[async_trait::async_trait]
    impl HttpCommand for GetFileLinkCommand {
        type Output = String;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let result: Response<Payload> = client
                .get_request("getfilelink", &self.identifier.to_http_params())
                .await?;
            result.payload().map(|res| res.to_url())
        }
    }
}

#[cfg(all(test, feature = "client-http"))]
mod http_tests {
    use super::GetFileLinkCommand;
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::region::Region;
    use mockito::Matcher;

    #[tokio::test]
    async fn success() {
        crate::tests::init();
        let mut server = mockito::Server::new_async().await;
        let m = server.mock("GET", "/getfilelink")
            .match_query(Matcher::AllOf(vec![
                                        Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                                        Matcher::UrlEncoded("fileid".into(), "42".into()),
            ]))
            .with_status(200)
            .with_body(r#"{
        "result": 0,
        "dwltag": "yvkNr0TqT6HFAWlVpdnHs5",
        "hash": 17869736033964340520,
        "size": 10485760,
        "expires": "Sat, 24 Jul 2021 03:18:31 +0000",
        "path": "\/DLZCAt2vXZejNfL5ZruLVZZTk2ev7Z2ZZNR5ZZdoz6ZXZQZZErw4bH0PfzBQt3LlgXMliXVtietX\/SAkdyBjkA7mQABbT.bin",
        "hosts": [
                "edef2.pcloud.com",
                "eu3.pcloud.com"
        ]
}"#)
.create();
        let creds = Credentials::AccessToken("access-token".into());
        let dc = Region::new(server.url());
        let api = HttpClient::new(creds, dc);
        let result = GetFileLinkCommand::new(42.into())
            .execute(&api)
            .await
            .unwrap();
        assert_eq!(result, "https://edef2.pcloud.com/DLZCAt2vXZejNfL5ZruLVZZTk2ev7Z2ZZNR5ZZdoz6ZXZQZZErw4bH0PfzBQt3LlgXMliXVtietX/SAkdyBjkA7mQABbT.bin");
        m.assert();
    }
}
