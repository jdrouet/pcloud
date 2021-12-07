use super::FolderResponse;
use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::entry::Folder;
use crate::error::Error;
use crate::http::HttpClient;
use crate::request::Response;

#[derive(Debug)]
pub struct Params {
    name: String,
    parent_id: u64,
    ignore_exists: bool,
}

impl Params {
    pub fn new<S: Into<String>>(name: S, parent_id: u64) -> Self {
        Self {
            name: name.into(),
            parent_id,
            ignore_exists: false,
        }
    }

    pub fn ignore_exists(mut self, value: bool) -> Self {
        self.ignore_exists = value;
        self
    }

    fn method(&self) -> &str {
        if self.ignore_exists {
            "createfolderifnotexists"
        } else {
            "createfolder"
        }
    }

    pub fn to_http_params(&self) -> Vec<(&str, String)> {
        vec![
            ("name", self.name.clone()),
            ("folderid", self.parent_id.to_string()),
        ]
    }

    pub fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        vec![
            ("name", BinaryValue::Text(self.name.clone())),
            ("folderid", BinaryValue::Number(self.parent_id)),
        ]
    }
}

impl HttpClient {
    /// Create a folder
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the folder.
    /// * `parent_id` - ID of the parent folder. Use 0 for the root folder.
    ///
    #[tracing::instrument(skip(self))]
    pub async fn create_folder(&self, params: &Params) -> Result<Folder, Error> {
        let result: Response<FolderResponse> = self
            .get_request(params.method(), &params.to_http_params())
            .await?;
        result.payload().map(|item| item.metadata)
    }
}

impl BinaryClient {
    #[tracing::instrument(skip(self))]
    pub fn create_folder(&mut self, params: &Params) -> Result<Folder, Error> {
        let result = self.send_command(params.method(), &params.to_binary_params())?;
        let result: Response<FolderResponse> = serde_json::from_value(result)?;
        result.payload().map(|item| item.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::Params;
    use crate::binary::BinaryClient;
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
    use crate::region::Region;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn success() {
        crate::tests::init();
        let m = mock("GET", "/createfolder")
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
        let creds = Credentials::AccessToken("access-token".into());
        let dc = Region::mock();
        let api = HttpClient::new(creds, dc);
        let result = api.create_folder(&Params::new("testing", 0)).await.unwrap();
        assert_eq!(result.base.name, "testing");
        m.assert();
    }

    #[tokio::test]
    async fn error() {
        crate::tests::init();
        let m = mock("GET", "/createfolder")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
                Matcher::UrlEncoded("name".into(), "testing".into()),
            ]))
            .with_status(200)
            .with_body(r#"{ "result": 1020, "error": "something went wrong" }"#)
            .create();
        let creds = Credentials::AccessToken("access-token".into());
        let dc = Region::mock();
        let api = HttpClient::new(creds, dc);
        let error = api
            .create_folder(&Params::new("testing", 0))
            .await
            .unwrap_err();
        assert!(matches!(error, crate::error::Error::Protocol(_, _)));
        m.assert();
    }

    #[test]
    #[ignore]
    fn binary_success() {
        let name = crate::tests::random_name();
        let mut client = BinaryClient::new(Credentials::from_env(), Region::eu()).unwrap();
        let res = client
            .create_folder(&Params::new(name.as_str(), 0))
            .unwrap();
        assert_eq!(res.base.name, name);
    }
}
