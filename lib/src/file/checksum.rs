use super::{File, FileIdentifier};

#[derive(Debug, serde::Deserialize)]
pub struct CheckSumFile {
    pub md5: Option<String>,
    pub sha256: Option<String>,
    pub sha1: String,
    pub metadata: File,
}

impl crate::Client {
    pub async fn get_file_checksum(
        &self,
        identifier: impl Into<FileIdentifier<'_>>,
    ) -> crate::Result<CheckSumFile> {
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
        let client = Client::new(server.url(), Credentials::access_token("access-token"));
        let result = client.get_file_checksum(42).await.unwrap();
        assert_eq!(
            result.sha256.unwrap(),
            "d535d3354f9d36741e311ac0855c5cde1e8e90eae947f320469f17514d182e19"
        );
        m.assert();
    }
}
