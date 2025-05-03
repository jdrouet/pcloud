use std::borrow::Cow;

use super::{File, FileIdentifier, FileResponse};

#[derive(serde::Serialize)]
struct FileRenameParams<'a> {
    #[serde(flatten)]
    identifier: FileIdentifier<'a>,
    #[serde(rename = "toname")]
    to_name: Cow<'a, str>,
}

impl crate::Client {
    pub async fn rename_file<'a>(
        &self,
        identifier: impl Into<FileIdentifier<'a>>,
        name: impl Into<Cow<'a, str>>,
    ) -> crate::Result<File> {
        self.get_request::<FileResponse, _>(
            "renamefile",
            FileRenameParams {
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
            .mock("GET", "/renamefile")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("fileid".into(), "42".into()),
                Matcher::UrlEncoded("toname".into(), "yolo.bin".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
    "result": 0,
    "metadata": {
        "name": "yolo.bin",
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
        let result = client.rename_file(42, "yolo.bin").await.unwrap();
        assert_eq!(result.file_id, 42);
        m.assert();
    }
}
