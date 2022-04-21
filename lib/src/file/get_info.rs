use super::FileIdentifier;
use crate::binary::BinaryClient;
use crate::entry::File;
use crate::error::Error;
use crate::http::HttpClient;
use crate::prelude::HttpCommand;
use crate::request::Response;

#[derive(Debug)]
pub struct FileCheckSumCommand {
    identifier: FileIdentifier,
}

impl FileCheckSumCommand {
    pub fn new(identifier: FileIdentifier) -> Self {
        Self { identifier }
    }
}

#[async_trait::async_trait(?Send)]
impl HttpCommand for FileCheckSumCommand {
    type Output = CheckSumFile;

    async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
        let result: Response<CheckSumFile> = client
            .get_request("checksumfile", &self.identifier.to_http_params())
            .await?;
        result.payload()
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct CheckSumFile {
    pub sha256: String,
    pub sha1: String,
    pub metadata: File,
}

impl BinaryClient {
    #[tracing::instrument(skip(self))]
    pub fn get_info_file<I: Into<FileIdentifier> + std::fmt::Debug>(
        &mut self,
        identifier: I,
    ) -> Result<CheckSumFile, Error> {
        let params: FileIdentifier = identifier.into();
        let result = self.send_command("checksumfile", &params.to_binary_params())?;
        let result: Response<CheckSumFile> = serde_json::from_value(result)?;
        result.payload()
    }
}

#[cfg(test)]
mod tests {
    use super::FileCheckSumCommand;
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::region::Region;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn success() {
        crate::tests::init();
        let m = mock("GET", "/checksumfile")
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
        let result = FileCheckSumCommand::new(42.into())
            .execute(&api)
            .await
            .unwrap();
        assert_eq!(
            result.sha256,
            "d535d3354f9d36741e311ac0855c5cde1e8e90eae947f320469f17514d182e19"
        );
        m.assert();
    }
}
