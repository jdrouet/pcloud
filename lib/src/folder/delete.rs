//! Resources needed to delete a folder

use super::FolderIdentifier;

#[derive(Debug, serde::Deserialize)]
pub struct RecursivePayload {
    #[serde(rename = "deletedfiles")]
    pub deleted_files: usize,
    #[serde(rename = "deletedfolders")]
    pub deleted_folders: usize,
}

/// Command to delete a folder
///
/// Executing this command will return a [`RecursivePayload`](RecursivePayload) on success.
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/folder/delete.html).
///
/// # Example using the [`HttpClient`](crate::client::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::client::HttpClientBuilder;
/// use pcloud::prelude::HttpCommand;
/// use pcloud::folder::delete::FolderDeleteCommand;
///
/// # tokio_test::block_on(async {
/// let client = HttpClientBuilder::from_env().build().unwrap();
/// let cmd = FolderDeleteCommand::new(12.into());
/// match cmd.execute(&client).await {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// # })
/// ```
#[derive(Debug)]
pub struct FolderDeleteCommand<'a> {
    pub identifier: FolderIdentifier<'a>,
    pub recursive: bool,
}

impl<'a> FolderDeleteCommand<'a> {
    pub fn new(identifier: FolderIdentifier<'a>) -> Self {
        Self {
            identifier,
            recursive: false,
        }
    }

    pub fn set_recursive(&mut self, value: bool) {
        self.recursive = value;
    }

    pub fn with_recursive(mut self, value: bool) -> Self {
        self.recursive = value;
        self
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::{FolderDeleteCommand, RecursivePayload};
    use crate::client::HttpClient;
    use crate::error::Error;
    use crate::folder::{FolderIdentifierParam, FolderResponse};
    use crate::prelude::HttpCommand;

    impl<'a> FolderDeleteCommand<'a> {
        async fn http_normal(self, client: &HttpClient) -> Result<RecursivePayload, Error> {
            let params = FolderIdentifierParam::from(self.identifier);
            client
                .get_request::<FolderResponse, _>("deletefolder", params)
                .await?;
            Ok(RecursivePayload {
                deleted_files: 0,
                deleted_folders: 1,
            })
        }

        async fn http_recursive(self, client: &HttpClient) -> Result<RecursivePayload, Error> {
            let params = FolderIdentifierParam::from(self.identifier);
            client
                .get_request::<RecursivePayload, _>("deletefolderrecursive", params)
                .await
        }
    }

    #[async_trait::async_trait]
    impl<'a> HttpCommand for FolderDeleteCommand<'a> {
        type Output = RecursivePayload;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            if self.recursive {
                self.http_recursive(client).await
            } else {
                self.http_normal(client).await
            }
        }
    }
}

#[cfg(all(test, feature = "client-http"))]
mod http_tests {
    use super::FolderDeleteCommand;
    use crate::client::HttpClient;
    use crate::credentials::Credentials;
    use crate::prelude::HttpCommand;
    use crate::region::Region;
    use mockito::Matcher;

    #[tokio::test]
    async fn delete_folder_success() {
        crate::tests::init();
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("GET", "/deletefolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "42".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
    "result": 0,
    "metadata": {
        "name": "testing",
        "created": "Fri, 23 Jul 2021 19:39:09 +0000",
        "ismine": true,
        "thumb": false,
        "modified": "Fri, 23 Jul 2021 19:39:09 +0000",
        "isdeleted": true,
        "comments": 0,
        "id": "d1073906688",
        "isshared": false,
        "icon": "folder",
        "isfolder": true,
        "parentfolderid": 0,
        "folderid": 42
    }
}"#,
            )
            .create();
        let creds = Credentials::access_token("access-token");
        let dc = Region::new(server.url());
        let api = HttpClient::new(creds, dc);
        let result = FolderDeleteCommand::new(42.into())
            .execute(&api)
            .await
            .unwrap();
        assert_eq!(result.deleted_folders, 1);
        m.assert();
    }
}
