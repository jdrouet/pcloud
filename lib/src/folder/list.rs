use super::{FolderIdentifier, FolderResponse};
use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::entry::Folder;
use crate::error::Error;
use crate::http::HttpClient;
use crate::prelude::HttpCommand;
use crate::request::Response;

#[derive(Debug)]
pub struct FolderListCommand {
    identifier: FolderIdentifier,
    recursive: bool,
    show_deleted: bool,
    no_files: bool,
    no_shares: bool,
}

impl FolderListCommand {
    pub fn new(identifier: FolderIdentifier) -> Self {
        Self {
            identifier,
            recursive: false,
            show_deleted: false,
            no_files: false,
            no_shares: false,
        }
    }

    pub fn set_recursive(&mut self, value: bool) {
        self.recursive = value;
    }

    /// If is set full directory tree will be returned, which means that all directories will have contents filed.
    pub fn recursive(mut self, value: bool) -> Self {
        self.recursive = value;
        self
    }

    pub fn set_show_deleted(&mut self, value: bool) {
        self.show_deleted = value;
    }

    /// If is set, deleted files and folders that can be undeleted will be displayed.
    pub fn show_deleted(mut self, value: bool) -> Self {
        self.show_deleted = value;
        self
    }

    pub fn set_no_files(&mut self, value: bool) {
        self.no_files = value;
    }

    /// If is set, only the folder (sub)structure will be returned.
    pub fn no_files(mut self, value: bool) -> Self {
        self.no_files = value;
        self
    }

    pub fn set_no_shares(&mut self, value: bool) {
        self.no_shares = value;
    }

    /// If is set, only user's own folders and files will be displayed.
    pub fn no_shares(mut self, value: bool) -> Self {
        self.no_shares = value;
        self
    }

    fn to_http_params(&self) -> Vec<(&str, String)> {
        let mut res = self.identifier.to_http_params();
        if self.recursive {
            res.push(("recursive", "1".to_string()));
        }
        if self.show_deleted {
            res.push(("showdeleted", "1".to_string()));
        }
        if self.no_files {
            res.push(("no_files", "1".to_string()));
        }
        if self.no_shares {
            res.push(("no_shares", "1".to_string()));
        }
        res
    }

    fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        let mut res = self.identifier.to_binary_params();
        if self.recursive {
            res.push(("recursive", BinaryValue::Bool(true)));
        }
        if self.show_deleted {
            res.push(("showdeleted", BinaryValue::Bool(true)));
        }
        if self.no_files {
            res.push(("no_files", BinaryValue::Bool(true)));
        }
        if self.no_shares {
            res.push(("no_shares", BinaryValue::Bool(true)));
        }
        res
    }
}

#[async_trait::async_trait(?Send)]
impl HttpCommand for FolderListCommand {
    type Output = Folder;

    async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
        let result: Response<FolderResponse> = client
            .get_request("listfolder", &self.to_http_params())
            .await?;
        result.payload().map(|item| item.metadata)
    }
}

impl BinaryClient {
    #[tracing::instrument(skip(self))]
    pub fn list_folder(&mut self, params: &FolderListCommand) -> Result<Folder, Error> {
        let result = self.send_command("listfolder", &params.to_binary_params())?;
        let result: Response<FolderResponse> = serde_json::from_value(result)?;
        result.payload().map(|res| res.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::FolderListCommand;
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::region::Region;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn success() {
        crate::tests::init();
        let m = mock("GET", "/listfolder")
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
        let creds = Credentials::AccessToken("access-token".into());
        let dc = Region::mock();
        let api = HttpClient::new(creds, dc);
        let payload = FolderListCommand::new(0.into())
            .execute(&api)
            .await
            .unwrap();
        assert_eq!(payload.base.parent_folder_id, Some(0));
        m.assert();
    }

    #[tokio::test]
    async fn error() {
        crate::tests::init();
        let m = mock("GET", "/listfolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
            ]))
            .with_status(200)
            .with_body(r#"{ "result": 1020, "error": "something went wrong" }"#)
            .create();
        let creds = Credentials::AccessToken("access-token".into());
        let dc = Region::mock();
        let api = HttpClient::new(creds, dc);
        let error = FolderListCommand::new(0.into())
            .execute(&api)
            .await
            .unwrap_err();
        assert!(matches!(error, crate::error::Error::Protocol(_, _)));
        m.assert();
    }

    #[test]
    #[cfg(feature = "protected")]
    fn binary_success() {
        use crate::binary::BinaryClient;

        let mut client = BinaryClient::new(Credentials::from_env(), Region::eu()).unwrap();
        let res = client.list_folder(&FolderListCommand::new(0)).unwrap();
        assert_eq!(res.base.name, "/");
    }
}
