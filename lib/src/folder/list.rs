use super::{Folder, FolderIdentifier, FolderResponse};

/// Options for customizing folder listing behavior.
///
/// This struct allows you to control recursion, visibility of deleted items,
/// and whether to include files and shared items in the response from `listfolder`.
#[derive(Default, serde::Serialize)]
pub struct ListFolderOptions {
    /// Whether to list the contents of subfolders recursively.
    #[serde(
        skip_serializing_if = "crate::request::is_false",
        serialize_with = "crate::request::serialize_bool"
    )]
    recursive: bool,

    /// Whether to include deleted files and folders in the listing.
    #[serde(
        rename = "showdeleted",
        skip_serializing_if = "crate::request::is_false",
        serialize_with = "crate::request::serialize_bool"
    )]
    show_deleted: bool,

    /// Whether to exclude files from the listing (only folders will be returned).
    #[serde(
        skip_serializing_if = "crate::request::is_false",
        serialize_with = "crate::request::serialize_bool"
    )]
    no_files: bool,

    /// Whether to exclude shared items from the listing.
    #[serde(
        skip_serializing_if = "crate::request::is_false",
        serialize_with = "crate::request::serialize_bool"
    )]
    no_shares: bool,
}

impl ListFolderOptions {
    /// Enables recursive listing of folder contents.
    pub fn with_recursive(mut self) -> Self {
        self.recursive = true;
        self
    }

    /// Enables showing deleted files and folders in the response.
    pub fn with_show_deleted(mut self) -> Self {
        self.show_deleted = true;
        self
    }

    /// Excludes files from the listing (folders only).
    pub fn with_no_files(mut self) -> Self {
        self.no_files = true;
        self
    }

    /// Excludes shared items from the listing.
    pub fn with_no_shares(mut self) -> Self {
        self.no_shares = true;
        self
    }
}

/// Internal parameter bundle for listing folders.
#[derive(serde::Serialize)]
struct Params<'a> {
    #[serde(flatten)]
    identifier: FolderIdentifier<'a>,
    #[serde(flatten)]
    options: ListFolderOptions,
}

impl crate::Client {
    /// Lists the contents of a folder on pCloud.
    ///
    /// This is a convenience method that calls [`crate::Client::list_folder_with_options`] with default options.
    /// It will list the folder's immediate contents, including files and subfolders.
    ///
    /// # Arguments
    ///
    /// * `identifier` - A value convertible into a [`FolderIdentifier`] (e.g., path or folder ID).
    ///
    /// # Returns
    ///
    /// A [`Folder`] struct containing metadata and child entries.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::Error`] if the folder cannot be listed.
    ///
    /// # Examples
    ///
    /// ```
    /// # async fn example(client: &pcloud::Client) -> Result<(), pcloud::Error> {
    /// let folder = client.list_folder("/Documents").await?;
    /// println!("Folder has {:?} entries", folder.contents.map(|res| res.len()).unwrap_or(0));
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_folder(
        &self,
        identifier: impl Into<FolderIdentifier<'_>>,
    ) -> crate::Result<Folder> {
        self.list_folder_with_options(identifier, Default::default())
            .await
    }

    /// Lists the contents of a folder with custom options.
    ///
    /// This method gives fine-grained control over how the folder contents are returned
    /// by the `listfolder` endpoint, such as recursive listing and filtering.
    ///
    /// # Arguments
    ///
    /// * `identifier` - A value convertible into a [`FolderIdentifier`] (e.g., path or folder ID).
    /// * `options` - A [`ListFolderOptions`] struct specifying what to include in the listing.
    ///
    /// # Returns
    ///
    /// A [`Folder`] with metadata and contents matching the provided options.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::Error`] if the folder is inaccessible or the API call fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// use pcloud::folder::list::ListFolderOptions;
    ///
    /// # async fn example(client: &pcloud::Client) -> Result<(), pcloud::Error> {
    /// let options = ListFolderOptions::default()
    ///     .with_recursive()
    ///     .with_show_deleted();
    /// let folder = client.list_folder_with_options("/Backup", options).await?;
    /// println!("Listed folder: {:?}", folder.base.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn list_folder_with_options(
        &self,
        identifier: impl Into<FolderIdentifier<'_>>,
        options: ListFolderOptions,
    ) -> crate::Result<Folder> {
        let params = Params {
            identifier: identifier.into(),
            options,
        };
        self.get_request::<FolderResponse, _>("listfolder", params)
            .await
            .map(|res| res.metadata)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Client, Credentials};
    use mockito::Matcher;

    #[tokio::test]
    async fn success_with_options() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("GET", "/listfolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
                Matcher::UrlEncoded("recursive".into(), "1".into()),
                Matcher::UrlEncoded("showdeleted".into(), "1".into()),
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
        let payload = client
            .list_folder_with_options(
                0,
                super::ListFolderOptions::default()
                    .with_recursive()
                    .with_show_deleted(),
            )
            .await
            .unwrap();
        assert_eq!(payload.base.parent_folder_id, Some(0));
        m.assert();
    }

    #[tokio::test]
    async fn success() {
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
        let client = Client::new(server.url(), Credentials::access_token("access-token")).unwrap();
        let payload = client.list_folder(0).await.unwrap();
        assert_eq!(payload.base.parent_folder_id, Some(0));
        m.assert();
    }

    #[tokio::test]
    async fn error() {
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
        let client = Client::new(server.url(), Credentials::access_token("access-token")).unwrap();
        let error = client.list_folder(0).await.unwrap_err();
        assert!(matches!(error, crate::Error::Protocol(_, _)));
        m.assert();
    }
}
