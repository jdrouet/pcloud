use super::{FolderIdentifier, FolderResponse};
use crate::binary::{PCloudBinaryApi, Value as BinaryValue};
use crate::entry::Folder;
use crate::error::Error;
use crate::http::HttpClient;
use crate::request::Response;

#[derive(Debug)]
pub struct Params {
    identifier: FolderIdentifier,
    recursive: bool,
    show_deleted: bool,
    no_files: bool,
    no_shares: bool,
}

impl Params {
    pub fn new<I: Into<FolderIdentifier>>(identifier: I) -> Self {
        Self {
            identifier: identifier.into(),
            recursive: false,
            show_deleted: false,
            no_files: false,
            no_shares: false,
        }
    }

    /// If is set full directory tree will be returned, which means that all directories will have contents filed.
    pub fn recursive(mut self, value: bool) -> Self {
        self.recursive = value;
        self
    }

    /// If is set, deleted files and folders that can be undeleted will be displayed.
    pub fn show_deleted(mut self, value: bool) -> Self {
        self.show_deleted = value;
        self
    }

    /// If is set, only the folder (sub)structure will be returned.
    pub fn no_files(mut self, value: bool) -> Self {
        self.no_files = value;
        self
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

impl HttpClient {
    /// List a folder's content
    ///
    /// # Arguments
    ///
    /// * `folder_id` - ID of the folder.
    ///
    pub async fn list_folder(&self, params: &Params) -> Result<Folder, Error> {
        let result: Response<FolderResponse> = self
            .get_request("listfolder", &params.to_http_params())
            .await?;
        result.payload().map(|item| item.metadata)
    }
}

impl PCloudBinaryApi {
    pub fn list_folder(&mut self, params: &Params) -> Result<Folder, Error> {
        let result = self.send_command("listfolder", &params.to_binary_params(), false, 0)?;
        let result: Response<FolderResponse> = serde_json::from_value(result)?;
        result.payload().map(|res| res.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::Params;
    use crate::binary::PCloudBinaryApi;
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
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
        let dc = Region::Test;
        let api = HttpClient::new(creds, dc);
        let payload = api.list_folder(&Params::new(0)).await.unwrap();
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
        let dc = Region::Test;
        let api = HttpClient::new(creds, dc);
        let error = api.list_folder(&Params::new(0)).await.unwrap_err();
        assert!(matches!(error, crate::error::Error::Payload(_, _)));
        m.assert();
    }

    #[test]
    fn binary_success() {
        let mut client = PCloudBinaryApi::new(Region::Europe, Credentials::from_env()).unwrap();
        let res = client.list_folder(&Params::new(0)).unwrap();
        assert_eq!(res.base.name, "/");
    }
}
