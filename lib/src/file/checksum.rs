use super::{File, FileIdentifier};

/// Represents the checksum information for a file stored on pCloud.
///
/// This struct includes various checksum types (MD5, SHA-1, SHA-256) as well
/// as metadata about the file itself.
#[derive(Debug, serde::Deserialize)]
pub struct FileChecksum {
    pub md5: Option<String>,
    pub sha256: Option<String>,
    pub sha1: String,
    pub metadata: File,
}

impl crate::Client {
    /// Retrieves the checksums and metadata for a file on pCloud.
    ///
    /// This function calls the `checksumfile` endpoint and returns a
    /// [`FileChecksum`] struct containing the MD5, SHA-1, and SHA-256 hashes,
    /// as well as basic metadata for the specified file.
    ///
    /// # Arguments
    ///
    /// * `identifier` - A type that can be converted into a [`FileIdentifier`] used to identify the file (e.g., by file ID or path).
    ///
    /// # Errors
    ///
    /// Returns a [`crate::Error`] if the request fails or if the file cannot be found.
    ///
    /// # Examples
    ///
    /// ```rust,no_run
    /// # async fn example(client: &pcloud::Client) -> Result<(), pcloud::Error> {
    /// let checksum = client.get_file_checksum("myfolder/myfile.txt").await?;
    /// println!("SHA-1: {}", checksum.sha1);
    /// # Ok(())
    /// # }
    /// ```
    pub async fn get_file_checksum(
        &self,
        identifier: impl Into<FileIdentifier<'_>>,
    ) -> crate::Result<FileChecksum> {
        self.get_request("checksumfile", identifier.into()).await
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
            .mock("GET", "/checksumfile")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("fileid".into(), "42".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
    "result": 0,
    "sha256": "d535d3354f9d36741e311ac0855c5cde1e8e90eae947f320469f17514d182e19",
    "sha1": "5b03ef4fa47ed13f2156ec5395866dadbde4e9dc",
    "metadata": {
        "name": "C61EWBrr2sU16GM4.bin",
        "created": "Sat, 24 Jul 2021 07:38:41 +0000",
        "thumb": false,
        "modified": "Sat, 24 Jul 2021 07:38:41 +0000",
        "isfolder": false,
        "fileid": 5257731387,
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
        let result = client.get_file_checksum(42).await.unwrap();
        assert_eq!(
            result.sha256.unwrap(),
            "d535d3354f9d36741e311ac0855c5cde1e8e90eae947f320469f17514d182e19"
        );
        m.assert();
    }
}
