use std::borrow::Cow;

use super::{Folder, FolderIdentifier, FolderResponse};

/// Parameters used for folder creation requests.
///
/// This struct is used internally to send a `parent` folder and `name`
/// to the `createfolder` or `createfolderifnotexists` API endpoints.
#[derive(Debug, serde::Serialize)]
struct Params<'a> {
    /// The parent folder in which the new folder will be created.
    #[serde(flatten)]
    parent: FolderIdentifier<'a>,

    /// The name of the new folder.
    name: Cow<'a, str>,
}

impl crate::Client {
    /// Creates a new folder inside the specified parent folder on pCloud.
    ///
    /// This function calls the `createfolder` API endpoint and will return
    /// an error if a folder with the same name already exists in the target location.
    ///
    /// # Arguments
    ///
    /// * `parent` - A value convertible into a [`FolderIdentifier`] representing the parent folder.
    /// * `name` - The name of the folder to create.
    ///
    /// # Returns
    ///
    /// On success, returns a [`Folder`] representing the newly created folder.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::Error`] if the folder already exists or the request fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example(client: &pcloud::Client) -> Result<(), pcloud::Error> {
    /// let folder = client.create_folder(0, "new-folder").await?;
    /// println!("Created folder: {}", folder.base.name);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_folder<'a>(
        &self,
        parent: impl Into<FolderIdentifier<'a>>,
        name: impl Into<Cow<'a, str>>,
    ) -> crate::Result<Folder> {
        let params = Params {
            parent: parent.into(),
            name: name.into(),
        };
        self.get_request::<FolderResponse, _>("createfolder", params)
            .await
            .map(|res| res.metadata)
    }
}

impl crate::Client {
    /// Creates a new folder if it does not already exist in the specified location.
    ///
    /// This function calls the `createfolderifnotexists` API endpoint, which ensures
    /// the operation is idempotent: if a folder with the given name exists, it will be returned.
    /// Otherwise, a new folder is created.
    ///
    /// # Arguments
    ///
    /// * `parent` - A value convertible into a [`FolderIdentifier`] representing the parent folder.
    /// * `name` - The name of the folder to create or return if it exists.
    ///
    /// # Returns
    ///
    /// A [`Folder`] representing the existing or newly created folder.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::Error`] if the request fails for any reason other than the folder already existing.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example(client: &pcloud::Client) -> Result<(), pcloud::Error> {
    /// let folder = client.create_folder_if_not_exists(0, "my-folder").await?;
    /// println!("Folder ID: {}", folder.folder_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn create_folder_if_not_exists<'a>(
        &self,
        parent: impl Into<FolderIdentifier<'a>>,
        name: impl Into<Cow<'a, str>>,
    ) -> crate::Result<Folder> {
        let params = Params {
            parent: parent.into(),
            name: name.into(),
        };
        self.get_request::<FolderResponse, _>("createfolderifnotexists", params)
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
            .mock("GET", "/createfolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
                Matcher::UrlEncoded("name".into(), "testing".into()),
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
        let result = client.create_folder(0, "testing").await.unwrap();
        assert_eq!(result.base.name, "testing");
        m.assert();
    }

    #[tokio::test]
    async fn error() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("GET", "/createfolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
                Matcher::UrlEncoded("name".into(), "testing".into()),
            ]))
            .with_status(200)
            .with_body(r#"{ "result": 1020, "error": "something went wrong" }"#)
            .create();
        let client = Client::new(server.url(), Credentials::access_token("access-token")).unwrap();
        let error = client.create_folder(0, "testing").await.unwrap_err();
        assert!(matches!(error, crate::Error::Protocol(_, _)));
        m.assert();
    }
}
