use std::borrow::Cow;

use super::{Folder, FolderIdentifier, FolderResponse};

#[derive(serde::Serialize)]
struct FolderRenameParams<'a> {
    #[serde(flatten)]
    identifier: FolderIdentifier<'a>,
    #[serde(rename = "toname")]
    to_name: Cow<'a, str>,
}

impl crate::Client {
    pub async fn rename_folder<'a>(
        &self,
        identifier: impl Into<FolderIdentifier<'a>>,
        name: impl Into<Cow<'a, str>>,
    ) -> crate::Result<Folder> {
        self.get_request::<FolderResponse, _>(
            "renamefolder",
            FolderRenameParams {
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
            .mock("GET", "/renamefolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "42".into()),
                Matcher::UrlEncoded("toname".into(), "yolo".into()),
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
        let result = client.rename_folder(42, "yolo").await.unwrap();
        assert_eq!(result.folder_id, 42);
        m.assert();
    }
}
