use std::borrow::Cow;

use super::StreamingLinkList;
use crate::file::FileIdentifier;

#[derive(Debug, Default, serde::Serialize)]
pub struct GetFileLinkParams<'a> {
    #[serde(
        rename = "forcedownload",
        skip_serializing_if = "crate::request::is_false",
        serialize_with = "crate::request::serialize_bool"
    )]
    force_download: bool,
    #[serde(rename = "contenttype", skip_serializing_if = "Option::is_none")]
    content_type: Option<Cow<'a, str>>,
    #[serde(rename = "maxspeed", skip_serializing_if = "Option::is_none")]
    max_speed: Option<u64>,
    #[serde(
        rename = "skip_filename",
        skip_serializing_if = "crate::request::is_false",
        serialize_with = "crate::request::serialize_bool"
    )]
    skip_filename: bool,
}

impl<'a> GetFileLinkParams<'a> {
    pub fn set_force_download(&mut self, value: bool) {
        self.force_download = value;
    }

    pub fn with_force_download(mut self, value: bool) -> Self {
        self.set_force_download(value);
        self
    }

    pub fn set_content_type(&mut self, value: impl Into<Cow<'a, str>>) {
        self.content_type = Some(value.into());
    }

    pub fn with_content_type(mut self, value: impl Into<Cow<'a, str>>) -> Self {
        self.set_content_type(value);
        self
    }

    pub fn set_max_speed(&mut self, value: u64) {
        self.max_speed = Some(value);
    }

    pub fn with_max_speed(mut self, value: u64) -> Self {
        self.set_max_speed(value);
        self
    }

    pub fn set_skip_filename(&mut self, value: bool) {
        self.skip_filename = value;
    }

    pub fn with_skip_filename(mut self, value: bool) -> Self {
        self.set_skip_filename(value);
        self
    }
}

#[derive(serde::Serialize)]
struct Params<'a> {
    #[serde(flatten)]
    identifier: FileIdentifier<'a>,
    #[serde(flatten)]
    params: GetFileLinkParams<'a>,
}

impl crate::Client {
    pub async fn get_file_link(
        &self,
        identifier: impl Into<FileIdentifier<'_>>,
    ) -> crate::Result<StreamingLinkList> {
        self.get_request::<StreamingLinkList, _>("getfilelink", identifier.into())
            .await
    }

    pub async fn get_file_link_with_params(
        &self,
        identifier: impl Into<FileIdentifier<'_>>,
        params: GetFileLinkParams<'_>,
    ) -> crate::Result<StreamingLinkList> {
        self.get_request::<StreamingLinkList, _>(
            "getfilelink",
            Params {
                identifier: identifier.into(),
                params,
            },
        )
        .await
    }
}

#[cfg(test)]
mod tests {
    use crate::{Client, Credentials};
    use mockito::Matcher;

    #[tokio::test]
    async fn success() {
        let mut server = mockito::Server::new_async().await;
        let m = server.mock("GET", "/getfilelink")
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
        let client = Client::new(server.url(), Credentials::access_token("access-token")).unwrap();
        let result = client.get_file_link(42).await.unwrap();
        let mut iter = result.links();
        assert_eq!(iter.next().unwrap().to_string(), "https://edef2.pcloud.com/DLZCAt2vXZejNfL5ZruLVZZTk2ev7Z2ZZNR5ZZdoz6ZXZQZZErw4bH0PfzBQt3LlgXMliXVtietX/SAkdyBjkA7mQABbT.bin");
        m.assert();
    }
}
