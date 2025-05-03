use crate::folder::{FolderIdentifier, ToFolderIdentifier};

use super::{File, FileIdentifier, FileResponse};

#[derive(serde::Serialize)]
struct FileMoveParams<'a> {
    #[serde(flatten)]
    from: FileIdentifier<'a>,
    #[serde(flatten)]
    to: ToFolderIdentifier<'a>,
}

impl crate::Client {
    pub async fn move_file(
        &self,
        file: impl Into<FileIdentifier<'_>>,
        to_folder: impl Into<FolderIdentifier<'_>>,
    ) -> crate::Result<File> {
        self.get_request::<FileResponse, _>(
            "renamefile",
            FileMoveParams {
                from: file.into(),
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
            .mock("GET", "/renamefile")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("fileid".into(), "42".into()),
                Matcher::UrlEncoded("topath".into(), "/this/dir/".into()),
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
        let result = client.move_file(42, "/this/dir/").await.unwrap();
        assert_eq!(result.file_id, 42);
        m.assert();
    }
}
