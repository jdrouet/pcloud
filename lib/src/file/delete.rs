use super::{File, FileIdentifier, FileResponse};

impl crate::Client {
    /// Deletes a file from pCloud.
    ///
    /// This function calls the `deletefile` endpoint to remove the specified file from the user's pCloud storage.
    /// It returns the metadata of the deleted file upon success.
    ///
    /// # Arguments
    ///
    /// * `identifier` - A type convertible into a [`FileIdentifier`] that identifies the file to delete
    /// (e.g., by file ID or path).
    ///
    /// # Returns
    ///
    /// A [`File`] struct containing metadata about the deleted file.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::Error`] if the file does not exist or the API request fails.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example(client: &pcloud::Client) -> Result<(), pcloud::Error> {
    /// let deleted_file = client.delete_file("myfolder/myfile.txt").await?;
    /// println!("Deleted file ID: {}", deleted_file.file_id);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn delete_file(
        &self,
        identifier: impl Into<FileIdentifier<'_>>,
    ) -> crate::Result<File> {
        self.get_request::<FileResponse, _>("deletefile", identifier.into())
            .await
            .map(|res| res.metadata)
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
            .mock("GET", "/deletefile")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("fileid".into(), "42".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
    "result": 0,
    "metadata": {
        "name": "C61EWBrr2sU16GM4.bin",
        "created": "Sat, 24 Jul 2021 07:38:41 +0000",
        "thumb": false,
        "modified": "Sat, 24 Jul 2021 07:38:41 +0000",
        "isfolder": false,
        "isdeleted": true,
        "fileid": 42,
        "hash": 9403476549337371523,
        "comments": 0,
        "category": 0,
        "id": "f5257731387",
        "isshared": false,
        "ismine": true,
        "size": 10485760,
        "parentfolderid": 1075398908,
        "contenttype": "application\/octet-stream",
        "icon": "file"
    }
}"#,
            )
            .create();
        let client = Client::new(server.url(), Credentials::access_token("access-token")).unwrap();
        let result = client.delete_file(42).await.unwrap();
        assert_eq!(result.file_id, 42);
        m.assert();
    }
}
