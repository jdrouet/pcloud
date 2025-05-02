use super::{Folder, FolderIdentifier, FolderResponse};

pub const RECURSIVE: u8 = 0b1;
pub const SHOW_DELETED: u8 = 0b10;
pub const NO_FILES: u8 = 0b100;
pub const NO_SHARES: u8 = 0b1000;

#[derive(serde::Serialize)]
struct FolderListParams<'a> {
    #[serde(flatten)]
    identifier: FolderIdentifier<'a>,
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

impl crate::Client {
    pub async fn list_folder(
        &self,
        identifier: impl Into<FolderIdentifier<'_>>,
    ) -> crate::Result<Folder> {
        self.list_folder_options(identifier, 0).await
    }

    pub async fn list_folder_options(
        &self,
        identifier: impl Into<FolderIdentifier<'_>>,
        options: u8,
    ) -> crate::Result<Folder> {
        let params = FolderListParams {
            identifier: identifier.into(),
            recursive: options & RECURSIVE > 0,
            show_deleted: options & SHOW_DELETED > 0,
            no_files: options & NO_FILES > 0,
            no_shares: options & NO_SHARES > 0,
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
        let client = Client::new(server.url(), Credentials::access_token("access-token"));
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
        let client = Client::new(server.url(), Credentials::access_token("access-token"));
        let error = client.list_folder(0).await.unwrap_err();
        assert!(matches!(error, crate::Error::Protocol(_, _)));
        m.assert();
    }
}
