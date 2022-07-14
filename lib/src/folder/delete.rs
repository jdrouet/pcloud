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
/// # Example using the [`HttpClient`](crate::http::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::http::HttpClientBuilder;
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
///
/// # Example using the [`BinaryClient`](crate::binary::BinaryClient)
///
/// To use this, the `client-binary` feature should be enabled.
///
/// ```
/// use pcloud::binary::BinaryClientBuilder;
/// use pcloud::prelude::BinaryCommand;
/// use pcloud::folder::delete::FolderDeleteCommand;
///
/// let mut client = BinaryClientBuilder::from_env().build().unwrap();
/// let cmd = FolderDeleteCommand::new("/foo/bar".into()).recursive(true);
/// match cmd.execute(&mut client) {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// ```
#[derive(Debug)]
pub struct FolderDeleteCommand {
    pub identifier: FolderIdentifier,
    pub recursive: bool,
}

impl FolderDeleteCommand {
    pub fn new(identifier: FolderIdentifier) -> Self {
        Self {
            identifier,
            recursive: false,
        }
    }

    pub fn recursive(mut self, value: bool) -> Self {
        self.recursive = value;
        self
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::{FolderDeleteCommand, RecursivePayload};
    use crate::error::Error;
    use crate::folder::FolderResponse;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::request::Response;

    impl FolderDeleteCommand {
        async fn http_normal(&self, client: &HttpClient) -> Result<RecursivePayload, Error> {
            let result: Response<FolderResponse> = client
                .get_request("deletefolder", &self.identifier.to_http_params())
                .await?;
            result.payload().map(|_| RecursivePayload {
                deleted_files: 0,
                deleted_folders: 1,
            })
        }

        async fn http_recursive(&self, client: &HttpClient) -> Result<RecursivePayload, Error> {
            let result: Response<RecursivePayload> = client
                .get_request("deletefolderrecursive", &self.identifier.to_http_params())
                .await?;
            result.payload()
        }
    }

    #[async_trait::async_trait]
    impl HttpCommand for FolderDeleteCommand {
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

#[cfg(feature = "client-binary")]
mod binary {
    use super::{FolderDeleteCommand, RecursivePayload};
    use crate::binary::BinaryClient;
    use crate::error::Error;
    use crate::folder::FolderResponse;
    use crate::prelude::BinaryCommand;
    use crate::request::Response;

    impl FolderDeleteCommand {
        fn binary_normal(&self, client: &mut BinaryClient) -> Result<RecursivePayload, Error> {
            let result =
                client.send_command("deletefolder", &self.identifier.to_binary_params())?;
            let result: Response<FolderResponse> = serde_json::from_value(result)?;
            result.payload().map(|_| RecursivePayload {
                deleted_files: 0,
                deleted_folders: 1,
            })
        }

        fn binary_recursive(&self, client: &mut BinaryClient) -> Result<RecursivePayload, Error> {
            let result = client
                .send_command("deletefolderrecursive", &self.identifier.to_binary_params())?;
            let result: Response<RecursivePayload> = serde_json::from_value(result)?;
            result.payload()
        }
    }

    impl BinaryCommand for FolderDeleteCommand {
        type Output = RecursivePayload;

        fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
            if self.recursive {
                self.binary_recursive(client)
            } else {
                self.binary_normal(client)
            }
        }
    }
}

#[cfg(all(test, feature = "client-http"))]
mod http_tests {
    use super::FolderDeleteCommand;
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::region::Region;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn delete_folder_success() {
        crate::tests::init();
        let m = mock("GET", "/deletefolder")
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
        let creds = Credentials::AccessToken("access-token".into());
        let dc = Region::mock();
        let api = HttpClient::new(creds, dc);
        let result = FolderDeleteCommand::new(42.into())
            .execute(&api)
            .await
            .unwrap();
        assert_eq!(result.deleted_folders, 1);
        m.assert();
    }
}
