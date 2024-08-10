//! Resources needed to list the content of a folder

use super::FolderIdentifier;

/// Command to list the content of a folder
///
/// Executing this command will return a [`Folder`](crate::entry::Folder) on success.
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/folder/listfolder.html).
///
/// # Example using the [`HttpClient`](crate::http::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::http::HttpClientBuilder;
/// use pcloud::prelude::HttpCommand;
/// use pcloud::folder::list::FolderListCommand;
///
/// # tokio_test::block_on(async {
/// let client = HttpClientBuilder::from_env().build().unwrap();
/// let cmd = FolderListCommand::new(0.into());
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
/// use pcloud::folder::list::FolderListCommand;
///
/// let mut client = BinaryClientBuilder::from_env().build().unwrap();
/// let cmd = FolderListCommand::new("/".into());
/// match cmd.execute(&mut client) {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// ```
#[derive(Debug)]
pub struct FolderListCommand {
    pub identifier: FolderIdentifier,
    pub recursive: bool,
    pub show_deleted: bool,
    pub no_files: bool,
    pub no_shares: bool,
}

impl FolderListCommand {
    pub fn new(identifier: FolderIdentifier) -> Self {
        Self {
            identifier,
            recursive: false,
            show_deleted: false,
            no_files: false,
            no_shares: false,
        }
    }

    /// If is set full directory tree will be returned, which means that all directories will have contents filed.
    pub fn recursive(mut self, value: bool) -> Self {
        self.recursive = value;
        self
    }

    /// If is set, deleted files and folders that can be undeleted will be displayed.
    pub fn show_deleted(mut self, value: bool) -> Self {
        self.show_deleted = value;
        self
    }

    /// If is set, only the folder (sub)structure will be returned.
    pub fn no_files(mut self, value: bool) -> Self {
        self.no_files = value;
        self
    }

    /// If is set, only user's own folders and files will be displayed.
    pub fn no_shares(mut self, value: bool) -> Self {
        self.no_shares = value;
        self
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::FolderListCommand;
    use crate::entry::Folder;
    use crate::error::Error;
    use crate::folder::FolderResponse;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::request::Response;

    impl FolderListCommand {
        fn to_http_params(&self) -> Vec<(&str, String)> {
            let mut res = self.identifier.to_http_params();
            if self.recursive {
                res.push(("recursive", "1".to_string()));
            }
            if self.show_deleted {
                res.push(("showdeleted", "1".to_string()));
            }
            if self.no_files {
                res.push(("no_files", "1".to_string()));
            }
            if self.no_shares {
                res.push(("no_shares", "1".to_string()));
            }
            res
        }
    }

    #[async_trait::async_trait]
    impl HttpCommand for FolderListCommand {
        type Output = Folder;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let result: Response<FolderResponse> = client
                .get_request("listfolder", &self.to_http_params())
                .await?;
            result.payload().map(|item| item.metadata)
        }
    }
}

#[cfg(all(test, feature = "client-http"))]
mod http_tests {
    use super::FolderListCommand;
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::region::Region;
    use mockito::Matcher;

    #[tokio::test]
    async fn success() {
        crate::tests::init();
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("GET", "/listfolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
    "result": 0,
    "metadata": {
        "path": "\/testing",
        "name": "testing",
        "created": "Fri, 23 Jul 2021 19:39:09 +0000",
        "ismine": true,
        "thumb": false,
        "modified": "Fri, 23 Jul 2021 19:39:09 +0000",
        "id": "d10",
        "isshared": false,
        "icon": "folder",
        "isfolder": true,
        "parentfolderid": 0,
        "folderid": 10
    }
}"#,
            )
            .create();
        let creds = Credentials::AccessToken("access-token".into());
        let dc = Region::new(server.url());
        let api = HttpClient::new(creds, dc);
        let payload = FolderListCommand::new(0.into())
            .execute(&api)
            .await
            .unwrap();
        assert_eq!(payload.base.parent_folder_id, Some(0));
        m.assert();
    }

    #[tokio::test]
    async fn error() {
        crate::tests::init();
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("GET", "/listfolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
            ]))
            .with_status(200)
            .with_body(r#"{ "result": 1020, "error": "something went wrong" }"#)
            .create();
        let creds = Credentials::AccessToken("access-token".into());
        let dc = Region::new(server.url());
        let api = HttpClient::new(creds, dc);
        let error = FolderListCommand::new(0.into())
            .execute(&api)
            .await
            .unwrap_err();
        assert!(matches!(error, crate::error::Error::Protocol(_, _)));
        m.assert();
    }
}
