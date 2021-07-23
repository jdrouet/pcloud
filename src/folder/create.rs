use super::FolderResponse;
use crate::common::RemoteFile;
use crate::request::{Error, Response};
use crate::PCloudApi;

impl PCloudApi {
    pub async fn create_folder(&self, name: &str, parent_id: usize) -> Result<RemoteFile, Error> {
        let parent_id = parent_id.to_string();
        let params = vec![("name", name), ("folderid", parent_id.as_str())];
        let result: Response<FolderResponse> = self.get_request("createfolder", &params).await?;
        result.payload().map(|item| item.metadata)
    }
}

#[cfg(test)]
mod tests {
    use crate::credentials::Credentials;
    use crate::data_center::DataCenter;
    use crate::PCloudApi;
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
        let dc = DataCenter::Test;
        let api = PCloudApi::new(creds, dc);
        let result = api.create_folder("testing", 0).await.unwrap();
        assert_eq!(result.name, "testing");
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
        let dc = DataCenter::Test;
        let api = PCloudApi::new(creds, dc);
        let error = api.create_folder("testing", 0).await.unwrap_err();
        assert!(matches!(error, crate::request::Error::Payload(_, _)));
        m.assert();
    }
}
