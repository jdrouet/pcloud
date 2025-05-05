use super::{Folder, FolderIdentifier, FolderResponse, ToFolderIdentifier};

/// Internal parameter structure for moving a folder.
#[derive(serde::Serialize)]
struct FolderMoveParams<'a> {
    /// The source folder identifier (either folder ID or path).
    #[serde(flatten)]
    from: FolderIdentifier<'a>,

    /// The destination folder identifier (either folder ID or path).
    #[serde(flatten)]
    to: ToFolderIdentifier<'a>,
}

impl crate::Client {
    /// Moves a folder to a new location in pCloud.
    ///
    /// This function calls the `renamefolder` API endpoint and moves the specified folder
    /// from its current location to the target folder. Both the source and destination can
    /// be specified either by folder ID or path.
    ///
    /// # Arguments
    ///
    /// * `folder` - A value that can be converted into a [`FolderIdentifier`] (e.g., folder ID or path).
    /// * `to_folder` - The target folder identifier (e.g., folder ID or path) to move the folder into.
    ///
    /// # Returns
    ///
    /// A [`Folder`] struct containing the metadata of the moved folder.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::Error`] if the move operation fails, such as if either folder is not accessible
    /// or the API call encounters an issue.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example(client: &pcloud::Client) -> Result<(), pcloud::Error> {
    /// let moved_folder = client.move_folder("/OldFolder", "/NewFolder").await?;
    /// println!("Moved folder: {:?}", moved_folder.base.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn move_folder(
        &self,
        folder: impl Into<FolderIdentifier<'_>>,
        to_folder: impl Into<FolderIdentifier<'_>>,
    ) -> crate::Result<Folder> {
        self.get_request::<FolderResponse, _>(
            "renamefolder",
            FolderMoveParams {
                from: folder.into(),
                to: ToFolderIdentifier(to_folder.into()),
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
                Matcher::UrlEncoded("topath".into(), "/this/dir/".into()),
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
        let result = client.move_folder(42, "/this/dir/").await.unwrap();
        assert_eq!(result.folder_id, 42);
        m.assert();
    }
}
