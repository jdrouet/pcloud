use super::{Folder, FolderIdentifier, FolderResponse};

/// Result payload from a recursive folder deletion request.
///
/// Returned by the `deletefolderrecursive` endpoint, it provides
/// information about how many files and folders were deleted in total.
#[derive(Debug, serde::Deserialize)]
pub struct RecursivePayload {
    /// The total number of files deleted.
    #[serde(rename = "deletedfiles")]
    pub deleted_files: usize,

    /// The total number of folders deleted.
    #[serde(rename = "deletedfolders")]
    pub deleted_folders: usize,
}

impl crate::Client {
    /// Deletes an empty folder from pCloud.
    ///
    /// This function calls the `deletefolder` API endpoint. It will fail
    /// if the folder is not empty or cannot be deleted.
    ///
    /// # Arguments
    ///
    /// * `identifier` - A value that can be converted into a [`FolderIdentifier`] (e.g., folder ID or path).
    ///
    /// # Returns
    ///
    /// On success, returns the metadata of the deleted folder as a [`Folder`] object.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::Error`] if the folder does not exist, is not empty, or the API call fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example(client: &pcloud::Client) -> Result<(), pcloud::Error> {
    /// let folder = client.delete_folder("/my/empty/folder").await?;
    /// println!("Deleted folder: {}", folder.base.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_folder<'a>(
        &self,
        identifier: impl Into<FolderIdentifier<'a>>,
    ) -> crate::Result<Folder> {
        self.get_request::<FolderResponse, _>("deletefolder", identifier.into())
            .await
            .map(|res| res.metadata)
    }
}

impl crate::Client {
    /// Recursively deletes a folder and all of its contents from pCloud.
    ///
    /// This function calls the `deletefolderrecursive` API endpoint and removes
    /// the folder and everything inside it (including subfolders and files).
    ///
    /// # Arguments
    ///
    /// * `identifier` - A value that can be converted into a [`FolderIdentifier`] (e.g., folder ID or path).
    ///
    /// # Returns
    ///
    /// A [`RecursivePayload`] struct containing statistics about how many files and folders were deleted.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::Error`] if the folder does not exist or the API call fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example(client: &pcloud::Client) -> Result<(), pcloud::Error> {
    /// let result = client.delete_folder_recursive(12345u64).await?;
    /// println!(
    ///     "Deleted {} files and {} folders",
    ///     result.deleted_files,
    ///     result.deleted_folders
    /// );
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_folder_recursive<'a>(
        &self,
        identifier: impl Into<FolderIdentifier<'a>>,
    ) -> crate::Result<RecursivePayload> {
        self.get_request("deletefolderrecursive", identifier.into())
            .await
    }
}

#[cfg(test)]
mod http_tests {
    use crate::{Client, Credentials};
    use mockito::Matcher;

    #[tokio::test]
    async fn success() {
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
        let client = Client::new(server.url(), Credentials::access_token("access-token")).unwrap();
        let result = client.delete_folder(42).await.unwrap();
        assert_eq!(result.base.name, "testing");
        m.assert();
    }
}
