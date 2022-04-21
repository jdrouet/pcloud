use super::FolderResponse;
use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::entry::Folder;
use crate::error::Error;
use crate::http::HttpClient;
use crate::prelude::HttpCommand;
use crate::request::Response;

#[derive(Debug)]
pub struct FolderCreateCommand {
    name: String,
    parent_id: u64,
    ignore_exists: bool,
}

impl FolderCreateCommand {
    pub fn new(name: String, parent_id: u64) -> Self {
        Self {
            name,
            parent_id,
            ignore_exists: false,
        }
    }

    pub fn set_ignore_exists(&mut self, value: bool) {
        self.ignore_exists = value;
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

#[async_trait::async_trait(?Send)]
impl HttpCommand for FolderCreateCommand {
    type Output = Folder;

    async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
        let result: Response<FolderResponse> = client
            .get_request(self.method(), &self.to_http_params())
            .await?;
        result.payload().map(|item| item.metadata)
    }
}

impl BinaryClient {
    #[tracing::instrument(skip(self))]
    pub fn create_folder(&mut self, params: &FolderCreateCommand) -> Result<Folder, Error> {
        let result = self.send_command(params.method(), &params.to_binary_params())?;
        let result: Response<FolderResponse> = serde_json::from_value(result)?;
        result.payload().map(|item| item.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::FolderCreateCommand;
    use crate::binary::BinaryClientBuilder;
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
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
        let result = FolderCreateCommand::new("testing".into(), 0)
            .execute(&api)
            .await
            .unwrap();
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
        let error = FolderCreateCommand::new("testing".into(), 0)
            .execute(&api)
            .await
            .unwrap_err();
        assert!(matches!(error, crate::error::Error::Protocol(_, _)));
        m.assert();
    }

    #[test]
    #[ignore]
    fn binary_success() {
        let name = crate::tests::random_name();
        let mut client = BinaryClientBuilder::from_env().build().unwrap();
        let res = client
            .create_folder(&FolderCreateCommand::new(name.clone(), 0))
            .unwrap();
        assert_eq!(res.base.name, name);
    }
}
