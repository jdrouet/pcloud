use super::FileIdentifier;
use crate::binary::BinaryClient;
use crate::entry::File;
use crate::error::Error;
use crate::file::FileResponse;
use crate::http::HttpClient;
use crate::prelude::HttpCommand;
use crate::request::Response;

#[derive(Debug)]
pub struct FileDeleteCommand {
    identifier: FileIdentifier,
}

impl FileDeleteCommand {
    pub fn new(identifier: FileIdentifier) -> Self {
        Self { identifier }
    }
}

#[async_trait::async_trait(?Send)]
impl HttpCommand for FileDeleteCommand {
    type Output = File;

    async fn execute(mut self, client: &HttpClient) -> Result<Self::Output, Error> {
        let result: Response<FileResponse> = client
            .get_request("deletefile", &self.identifier.to_http_params())
            .await?;
        result.payload().map(|res| res.metadata)
    }
}

impl BinaryClient {
    #[tracing::instrument(skip(self))]
    pub fn delete_file<I: Into<FileIdentifier> + std::fmt::Debug>(
        &mut self,
        identifier: I,
    ) -> Result<File, Error> {
        let identifier = identifier.into();
        let result = self.send_command("deletefile", &identifier.to_binary_params())?;
        let result: Response<FileResponse> = serde_json::from_value(result)?;
        result.payload().map(|res| res.metadata)
    }
}

#[cfg(test)]
mod tests {
    use super::FileDeleteCommand;
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::region::Region;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn success() {
        crate::tests::init();
        let m = mock("GET", "/deletefile")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("fileid".into(), "42".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
        "result": 0,
        "sha256": "d535d3354f9d36741e311ac0855c5cde1e8e90eae947f320469f17514d182e19",
        "sha1": "5b03ef4fa47ed13f2156ec5395866dadbde4e9dc",
        "metadata": {
                "name": "C61EWBrr2sU16GM4.bin",
                "created": "Sat, 24 Jul 2021 07:38:41 +0000",
                "thumb": false,
                "modified": "Sat, 24 Jul 2021 07:38:41 +0000",
                "isfolder": false,
                "fileid": 5257731387,
                "hash": 9403476549337371523,
                "comments": 0,
                "category": 0,
                "id": "f5257731387",
                "isshared": false,
                "ismine": true,
                "size": 10485760,
                "parentfolderid": 1075398908,
                "contenttype": "application\/octet-stream",
                "icon": "file"
        }
}"#,
            )
            .create();
        let creds = Credentials::AccessToken("access-token".into());
        let dc = Region::mock();
        let api = HttpClient::new(creds, dc);
        FileDeleteCommand::new(42.into())
            .execute(&api)
            .await
            .unwrap();
        m.assert();
    }
}
