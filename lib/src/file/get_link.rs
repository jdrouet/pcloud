use crate::error::Error;
use crate::request::Response;
use crate::PCloudApi;

#[derive(Debug, serde::Deserialize)]
struct Payload {
    // hash: usize,
    // size: usize,
    // expired: String,
    hosts: Vec<String>,
    path: String,
}

impl Payload {
    fn to_url(&self) -> String {
        let host = self.hosts.get(0).unwrap();
        format!("https://{}{}", host, self.path)
    }
}

impl PCloudApi {
    pub async fn get_link_file(&self, file_id: usize) -> Result<String, Error> {
        let file_id = file_id.to_string();
        let params = vec![("fileid", file_id.as_str())];
        let result: Response<Payload> = self.get_request("getfilelink", &params).await?;
        result.payload().map(|res| res.to_url())
    }
}

#[cfg(test)]
mod tests {
    use crate::credentials::Credentials;
    use crate::region::Region;
    use crate::PCloudApi;
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
        let dc = Region::Test;
        let api = PCloudApi::new(creds, dc);
        let result = api.get_link_file(42).await.unwrap();
        assert_eq!(result, "https://edef2.pcloud.com/DLZCAt2vXZejNfL5ZruLVZZTk2ev7Z2ZZNR5ZZdoz6ZXZQZZErw4bH0PfzBQt3LlgXMliXVtietX/SAkdyBjkA7mQABbT.bin");
        m.assert();
    }
}