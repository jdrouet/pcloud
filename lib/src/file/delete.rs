use super::{File, FileIdentifier, FileResponse};

#[derive(Debug, serde::Deserialize)]
pub struct FileChecksum {
    pub md5: Option<String>,
    pub sha256: Option<String>,
    pub sha1: String,
    pub metadata: File,
}

impl crate::Client {
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
