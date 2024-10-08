//! Resources needed to delete a file

use super::FileIdentifier;

/// Command to delete a file
///
/// Executing this command will return the deleted [`File`](crate::entry::File).
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/file/deletefile.html)
///
/// # Example using the [`HttpClient`](crate::client::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::client::HttpClientBuilder;
/// use pcloud::prelude::HttpCommand;
/// use pcloud::file::delete::FileDeleteCommand;
///
/// # tokio_test::block_on(async {
/// let client = HttpClientBuilder::from_env().build().unwrap();
/// let cmd = FileDeleteCommand::new("/foo/bar.txt".into());
/// match cmd.execute(&client).await {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// # })
/// ```

#[derive(Debug)]
pub struct FileDeleteCommand<'a> {
    pub identifier: FileIdentifier<'a>,
}

impl<'a> FileDeleteCommand<'a> {
    pub fn new(identifier: FileIdentifier<'a>) -> Self {
        Self { identifier }
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::FileDeleteCommand;
    use crate::client::HttpClient;
    use crate::entry::File;
    use crate::error::Error;
    use crate::file::FileIdentifierParam;
    use crate::file::FileResponse;
    use crate::prelude::HttpCommand;

    #[async_trait::async_trait]
    impl<'a> HttpCommand for FileDeleteCommand<'a> {
        type Output = File;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            client
                .get_request::<FileResponse, _>(
                    "deletefile",
                    FileIdentifierParam::from(self.identifier),
                )
                .await
                .map(|res| res.metadata)
        }
    }
}

#[cfg(all(test, feature = "client-http"))]
mod http_tests {
    use super::FileDeleteCommand;
    use crate::client::HttpClient;
    use crate::credentials::Credentials;
    use crate::prelude::HttpCommand;
    use crate::region::Region;
    use mockito::Matcher;

    #[tokio::test]
    async fn success() {
        crate::tests::init();
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("GET", "/deletefile")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("fileid".into(), "42".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
        "result": 0,
        "sha256": "d535d3354f9d36741e311ac0855c5cde1e8e90eae947f320469f17514d182e19",
        "sha1": "5b03ef4fa47ed13f2156ec5395866dadbde4e9dc",
        "metadata": {
                "name": "C61EWBrr2sU16GM4.bin",
                "created": "Sat, 24 Jul 2021 07:38:41 +0000",
                "thumb": false,
                "modified": "Sat, 24 Jul 2021 07:38:41 +0000",
                "isfolder": false,
                "fileid": 5257731387,
                "hash": 9403476549337371523,
                "comments": 0,
                "category": 0,
                "id": "f5257731387",
                "isshared": false,
                "ismine": true,
                "size": 10485760,
                "parentfolderid": 1075398908,
                "contenttype": "application\/octet-stream",
                "icon": "file"
        }
}"#,
            )
            .create();
        let creds = Credentials::access_token("access-token");
        let dc = Region::new(server.url());
        let api = HttpClient::new(creds, dc);
        FileDeleteCommand::new(42.into())
            .execute(&api)
            .await
            .unwrap();
        m.assert();
    }
}
