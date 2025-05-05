use std::borrow::Cow;

use super::{Folder, FolderIdentifier, FolderResponse};

/// Internal parameter structure for renaming a folder.
#[derive(serde::Serialize)]
struct FolderRenameParams<'a> {
    /// The folder identifier (either folder ID or path).
    #[serde(flatten)]
    identifier: FolderIdentifier<'a>,

    /// The new name for the folder.
    #[serde(rename = "toname")]
    to_name: Cow<'a, str>,
}

impl crate::Client {
    /// Renames an existing folder in pCloud.
    ///
    /// This function calls the `renamefolder` API endpoint to rename the specified folder.
    /// The folder is identified either by its folder ID or its path, and the new name is provided
    /// as a string.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier for the folder to be renamed, which can be provided either
    ///                  by folder ID or path.
    /// * `name` - The new name for the folder.
    ///
    /// # Returns
    ///
    /// A [`Folder`] struct containing the metadata of the renamed folder.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::Error`] if the rename operation fails, for example, if the folder does
    /// not exist or the API request encounters an issue.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example(client: &pcloud::Client) -> Result<(), pcloud::Error> {
    /// let renamed_folder = client.rename_folder("/OldFolder", "NewFolderName").await?;
    /// println!("Renamed folder: {:?}", renamed_folder.base.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn rename_folder<'a>(
        &self,
        identifier: impl Into<FolderIdentifier<'a>>,
        name: impl Into<Cow<'a, str>>,
    ) -> crate::Result<Folder> {
        self.get_request::<FolderResponse, _>(
            "renamefolder",
            FolderRenameParams {
                identifier: identifier.into(),
                to_name: name.into(),
            },
        )
        .await
        .map(|res| res.metadata)
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
            .mock("GET", "/renamefolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "42".into()),
                Matcher::UrlEncoded("toname".into(), "yolo".into()),
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
        "folderid": 42
    }
}"#,
            )
            .create();
        let client = Client::new(server.url(), Credentials::access_token("access-token")).unwrap();
        let result = client.rename_folder(42, "yolo").await.unwrap();
        assert_eq!(result.folder_id, 42);
        m.assert();
    }
}
