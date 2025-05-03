use super::{Folder, FolderIdentifier, FolderResponse, ToFolderIdentifier};

#[derive(serde::Serialize)]
struct FolderMoveParams<'a> {
    #[serde(flatten)]
    from: FolderIdentifier<'a>,
    #[serde(flatten)]
    to: ToFolderIdentifier<'a>,
}

impl crate::Client {
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
