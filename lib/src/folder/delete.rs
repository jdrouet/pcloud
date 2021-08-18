use super::{FolderIdentifier, FolderResponse};
use crate::entry::Folder;
use crate::error::Error;
use crate::http::PCloudApi;
use crate::request::Response;

impl PCloudApi {
    /// Delete a folder
    ///
    /// # Arguments
    ///
    /// * `folder_id` - ID of the folder to delete.
    ///
    pub async fn delete_folder<I: Into<FolderIdentifier>>(
        &self,
        identifier: I,
    ) -> Result<Folder, Error> {
        let identifier = identifier.into();
        let result: Response<FolderResponse> = self
            .get_request("deletefolder", &identifier.to_vec())
            .await?;
        result.payload().map(|item| item.metadata)
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct RecursivePayload {
    #[serde(rename = "deletedfiles")]
    pub deleted_files: usize,
    #[serde(rename = "deletedfolders")]
    pub deleted_folders: usize,
}

impl PCloudApi {
    /// Delete a folder recursively
    ///
    /// # Arguments
    ///
    /// * `folder_id` - ID of the folder to delete.
    ///
    pub async fn delete_folder_recursive<I: Into<FolderIdentifier>>(
        &self,
        identifier: I,
    ) -> Result<RecursivePayload, Error> {
        let identifier = identifier.into();
        let result: Response<RecursivePayload> = self
            .get_request("deletefolderrecursive", &identifier.to_vec())
            .await?;
        result.payload()
    }
}

#[cfg(test)]
mod tests {
    use crate::credentials::Credentials;
    use crate::http::PCloudApi;
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
        let dc = Region::Test;
        let api = PCloudApi::new(creds, dc);
        let result = api.delete_folder(42).await.unwrap();
        assert_eq!(result.base.name, "testing");
        m.assert();
    }
}
