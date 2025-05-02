use super::{Folder, FolderIdentifier, FolderResponse};

#[derive(Default, serde::Serialize)]
pub struct Options {
    #[serde(
        skip_serializing_if = "crate::request::is_false",
        serialize_with = "crate::request::serialize_bool"
    )]
    recursive: bool,
    #[serde(
        rename = "showdeleted",
        skip_serializing_if = "crate::request::is_false",
        serialize_with = "crate::request::serialize_bool"
    )]
    show_deleted: bool,
    #[serde(
        skip_serializing_if = "crate::request::is_false",
        serialize_with = "crate::request::serialize_bool"
    )]
    no_files: bool,
    #[serde(
        skip_serializing_if = "crate::request::is_false",
        serialize_with = "crate::request::serialize_bool"
    )]
    no_shares: bool,
}

impl Options {
    pub fn with_recursive(mut self) -> Self {
        self.recursive = true;
        self
    }

    pub fn with_show_deleted(mut self) -> Self {
        self.show_deleted = true;
        self
    }

    pub fn with_no_files(mut self) -> Self {
        self.no_files = true;
        self
    }

    pub fn with_no_shares(mut self) -> Self {
        self.no_shares = true;
        self
    }
}

#[derive(serde::Serialize)]
struct Params<'a> {
    #[serde(flatten)]
    identifier: FolderIdentifier<'a>,
    #[serde(flatten)]
    options: Options,
}

impl crate::Client {
    pub async fn list_folder(
        &self,
        identifier: impl Into<FolderIdentifier<'_>>,
    ) -> crate::Result<Folder> {
        self.list_folder_with_options(identifier, Default::default())
            .await
    }

    pub async fn list_folder_with_options(
        &self,
        identifier: impl Into<FolderIdentifier<'_>>,
        options: Options,
    ) -> crate::Result<Folder> {
        let params = Params {
            identifier: identifier.into(),
            options,
        };
        self.get_request::<FolderResponse, _>("listfolder", params)
            .await
            .map(|res| res.metadata)
    }
}

#[cfg(test)]
mod tests {
    use crate::{Client, Credentials};
    use mockito::Matcher;

    #[tokio::test]
    async fn success_with_options() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("GET", "/listfolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
                Matcher::UrlEncoded("recursive".into(), "1".into()),
                Matcher::UrlEncoded("showdeleted".into(), "1".into()),
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
        let payload = client
            .list_folder_with_options(
                0,
                super::Options::default()
                    .with_recursive()
                    .with_show_deleted(),
            )
            .await
            .unwrap();
        assert_eq!(payload.base.parent_folder_id, Some(0));
        m.assert();
    }

    #[tokio::test]
    async fn success() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("GET", "/listfolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
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
        let payload = client.list_folder(0).await.unwrap();
        assert_eq!(payload.base.parent_folder_id, Some(0));
        m.assert();
    }

    #[tokio::test]
    async fn error() {
        let mut server = mockito::Server::new_async().await;
        let m = server
            .mock("GET", "/listfolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
            ]))
            .with_status(200)
            .with_body(r#"{ "result": 1020, "error": "something went wrong" }"#)
            .create();
        let client = Client::new(server.url(), Credentials::access_token("access-token")).unwrap();
        let error = client.list_folder(0).await.unwrap_err();
        assert!(matches!(error, crate::Error::Protocol(_, _)));
        m.assert();
    }
}
