use std::borrow::Cow;

use super::{Folder, FolderIdentifier, FolderResponse};

#[derive(Debug, serde::Serialize)]
struct Params<'a> {
    #[serde(flatten)]
    parent: FolderIdentifier<'a>,
    name: Cow<'a, str>,
}

impl crate::Client {
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
        let client = Client::new(server.url(), Credentials::access_token("access-token"));
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
        let client = Client::new(server.url(), Credentials::access_token("access-token"));
        let error = client.create_folder(0, "testing").await.unwrap_err();
        assert!(matches!(error, crate::Error::Protocol(_, _)));
        m.assert();
    }
}
