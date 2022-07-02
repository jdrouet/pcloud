//! Resources needed to delete a file

use super::FileIdentifier;

/// Command to delete a file
///
/// Executing this command will return the deleted [`File`](crate::entry::File).
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/file/deletefile.html)
///
/// # Example using the [`HttpClient`](crate::http::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::http::HttpClientBuilder;
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
///
/// # Example using the [`BinaryClient`](crate::binary::BinaryClient)
///
/// To use this, the `client-binary` feature should be enabled.
///
/// ```
/// use pcloud::binary::BinaryClientBuilder;
/// use pcloud::prelude::BinaryCommand;
/// use pcloud::file::delete::FileDeleteCommand;
///
/// let mut client = BinaryClientBuilder::from_env().build().unwrap();
/// let cmd = FileDeleteCommand::new("/foo/bar.txt".into());
/// match cmd.execute(&mut client) {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// ```

#[derive(Debug)]
pub struct FileDeleteCommand {
    pub identifier: FileIdentifier,
}

impl FileDeleteCommand {
    pub fn new(identifier: FileIdentifier) -> Self {
        Self { identifier }
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::FileDeleteCommand;
    use crate::entry::File;
    use crate::error::Error;
    use crate::file::FileResponse;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::request::Response;

    #[async_trait::async_trait(?Send)]
    impl HttpCommand for FileDeleteCommand {
        type Output = File;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let result: Response<FileResponse> = client
                .get_request("deletefile", &self.identifier.to_http_params())
                .await?;
            result.payload().map(|res| res.metadata)
        }
    }
}

#[cfg(feature = "client-binary")]
mod binary {
    use super::FileDeleteCommand;
    use crate::binary::BinaryClient;
    use crate::entry::File;
    use crate::error::Error;
    use crate::file::FileResponse;
    use crate::prelude::BinaryCommand;
    use crate::request::Response;

    impl BinaryCommand for FileDeleteCommand {
        type Output = File;

        fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
            let result = client.send_command("deletefile", &self.identifier.to_binary_params())?;
            let result: Response<FileResponse> = serde_json::from_value(result)?;
            result.payload().map(|res| res.metadata)
        }
    }
}

#[cfg(all(test, feature = "client-http"))]
mod http_tests {
    use super::FileDeleteCommand;
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::region::Region;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn success() {
        crate::tests::init();
        let m = mock("GET", "/deletefile")
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
        let creds = Credentials::AccessToken("access-token".into());
        let dc = Region::mock();
        let api = HttpClient::new(creds, dc);
        FileDeleteCommand::new(42.into())
            .execute(&api)
            .await
            .unwrap();
        m.assert();
    }
}
