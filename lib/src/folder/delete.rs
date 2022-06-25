use super::{FolderIdentifier, FolderResponse};
use crate::binary::BinaryClient;
use crate::error::Error;
use crate::http::HttpClient;
use crate::prelude::{BinaryCommand, HttpCommand};
use crate::request::Response;

#[derive(Debug, serde::Deserialize)]
pub struct RecursivePayload {
    #[serde(rename = "deletedfiles")]
    pub deleted_files: usize,
    #[serde(rename = "deletedfolders")]
    pub deleted_folders: usize,
}

#[derive(Debug)]
pub struct FolderDeleteCommand {
    identifier: FolderIdentifier,
    recursive: bool,
}

impl FolderDeleteCommand {
    pub fn new(identifier: FolderIdentifier) -> Self {
        Self {
            identifier,
            recursive: false,
        }
    }

    pub fn set_recursive(&mut self, value: bool) {
        self.recursive = value;
    }

    pub fn recursive(mut self, value: bool) -> Self {
        self.recursive = value;
        self
    }

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

    fn binary_normal(&self, client: &mut BinaryClient) -> Result<RecursivePayload, Error> {
        let result = client.send_command("deletefolder", &self.identifier.to_binary_params())?;
        let result: Response<FolderResponse> = serde_json::from_value(result)?;
        result.payload().map(|_| RecursivePayload {
            deleted_files: 0,
            deleted_folders: 1,
        })
    }

    fn binary_recursive(&self, client: &mut BinaryClient) -> Result<RecursivePayload, Error> {
        let result =
            client.send_command("deletefolderrecursive", &self.identifier.to_binary_params())?;
        let result: Response<RecursivePayload> = serde_json::from_value(result)?;
        result.payload()
    }
}

#[async_trait::async_trait(?Send)]
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

#[cfg(test)]
mod tests {
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
