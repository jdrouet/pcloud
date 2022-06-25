use super::FileIdentifier;
use crate::error::Error;
use crate::http::HttpClient;
use crate::prelude::HttpCommand;
use crate::request::Response;

#[derive(Debug)]
pub struct FileLinkCommand {
    identifier: FileIdentifier,
}

impl FileLinkCommand {
    pub fn new(identifier: FileIdentifier) -> Self {
        Self { identifier }
    }
}

#[async_trait::async_trait(?Send)]
impl HttpCommand for FileLinkCommand {
    type Output = String;

    async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
        let result: Response<FileLink> = client
            .get_request("getfilelink", &self.identifier.to_http_params())
            .await?;
        result.payload().map(|res| res.to_url())
    }
}

#[derive(Debug, serde::Deserialize)]
struct FileLink {
    // hash: usize,
    // size: usize,
    // expired: String,
    hosts: Vec<String>,
    path: String,
}

impl FileLink {
    fn to_url(&self) -> String {
        let host = self.hosts.get(0).unwrap();
        format!("https://{}{}", host, self.path)
    }
}

#[cfg(test)]
mod tests {
    use super::FileLinkCommand;
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::region::Region;
    use mockito::{mock, Matcher};

    #[tokio::test]
    async fn success() {
        crate::tests::init();
        let m = mock("GET", "/getfilelink")
            .match_query(Matcher::AllOf(vec![
                                        Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                                        Matcher::UrlEncoded("fileid".into(), "42".into()),
            ]))
            .with_status(200)
            .with_body(r#"{
        "result": 0,
        "dwltag": "yvkNr0TqT6HFAWlVpdnHs5",
        "hash": 17869736033964340520,
        "size": 10485760,
        "expires": "Sat, 24 Jul 2021 03:18:31 +0000",
        "path": "\/DLZCAt2vXZejNfL5ZruLVZZTk2ev7Z2ZZNR5ZZdoz6ZXZQZZErw4bH0PfzBQt3LlgXMliXVtietX\/SAkdyBjkA7mQABbT.bin",
        "hosts": [
                "edef2.pcloud.com",
                "eu3.pcloud.com"
        ]
}"#)
.create();
        let creds = Credentials::AccessToken("access-token".into());
        let dc = Region::mock();
        let api = HttpClient::new(creds, dc);
        let result = FileLinkCommand::new(42.into()).execute(&api).await.unwrap();
        assert_eq!(result, "https://edef2.pcloud.com/DLZCAt2vXZejNfL5ZruLVZZTk2ev7Z2ZZNR5ZZdoz6ZXZQZZErw4bH0PfzBQt3LlgXMliXVtietX/SAkdyBjkA7mQABbT.bin");
        m.assert();
    }
}
