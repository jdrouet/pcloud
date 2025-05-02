use super::{Folder, FolderIdentifier, FolderResponse};

#[derive(Debug, serde::Deserialize)]
pub struct RecursivePayload {
    #[serde(rename = "deletedfiles")]
    pub deleted_files: usize,
    #[serde(rename = "deletedfolders")]
    pub deleted_folders: usize,
}

impl crate::Client {
    pub async fn delete_folder<'a>(
        &self,
        identifier: impl Into<FolderIdentifier<'a>>,
    ) -> crate::Result<Folder> {
        self.get_request::<FolderResponse, _>("deletefolder", identifier.into())
            .await
            .map(|res| res.metadata)
    }
}

impl crate::Client {
    pub async fn delete_folder_recursive<'a>(
        &self,
        identifier: impl Into<FolderIdentifier<'a>>,
    ) -> crate::Result<RecursivePayload> {
        self.get_request("deletefolderrecursive", identifier.into())
            .await
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
            .mock("GET", "/deletefolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "42".into()),
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
        let result = client.delete_folder(42).await.unwrap();
        assert_eq!(result.base.name, "testing");
        m.assert();
    }
}
