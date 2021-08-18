use super::FolderResponse;
use crate::entry::Folder;
use crate::error::Error;
use crate::http::PCloudApi;
use crate::request::Response;

#[derive(Debug)]
pub struct Params {
    name: String,
    parent_id: usize,
}

impl Params {
    pub fn new<S: Into<String>>(name: S, parent_id: usize) -> Self {
        Self {
            name: name.into(),
            parent_id,
        }
    }

    pub fn to_vec(&self) -> Vec<(&str, String)> {
        vec![
            ("name", self.name.clone()),
            ("folderid", self.parent_id.to_string()),
        ]
    }
}

impl PCloudApi {
    /// Create a folder
    ///
    /// # Arguments
    ///
    /// * `name` - Name of the folder.
    /// * `parent_id` - ID of the parent folder. Use 0 for the root folder.
    ///
    pub async fn create_folder(&self, params: &Params) -> Result<Folder, Error> {
        let result: Response<FolderResponse> =
            self.get_request("createfolder", &params.to_vec()).await?;
        result.payload().map(|item| item.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::Params;
    use crate::credentials::Credentials;
    use crate::http::PCloudApi;
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
        let dc = Region::Test;
        let api = PCloudApi::new(creds, dc);
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
        let dc = Region::Test;
        let api = PCloudApi::new(creds, dc);
        let error = api
            .create_folder(&Params::new("testing", 0))
            .await
            .unwrap_err();
        assert!(matches!(error, crate::error::Error::Payload(_, _)));
        m.assert();
    }
}
