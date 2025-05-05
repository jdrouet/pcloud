use std::borrow::Cow;

use super::StreamingLinkList;
use crate::file::FileIdentifier;

/// Parameters for retrieving a file link, including options for controlling the download behavior and file metadata.
#[derive(Debug, Default, serde::Serialize)]
pub struct GetFileLinkParams<'a> {
    /// Flag to force the file to be downloaded, rather than viewed in the browser.
    #[serde(
        rename = "forcedownload",
        skip_serializing_if = "crate::request::is_false",
        serialize_with = "crate::request::serialize_bool"
    )]
    force_download: bool,

    /// The content type of the file (e.g., "application/pdf"). This is optional.
    #[serde(rename = "contenttype", skip_serializing_if = "Option::is_none")]
    content_type: Option<Cow<'a, str>>,

    /// Maximum download speed (in bytes per second). This is optional.
    #[serde(rename = "maxspeed", skip_serializing_if = "Option::is_none")]
    max_speed: Option<u64>,

    /// Flag to skip the filename in the file link. If true, the filename is not included in the response URL.
    #[serde(
        rename = "skip_filename",
        skip_serializing_if = "crate::request::is_false",
        serialize_with = "crate::request::serialize_bool"
    )]
    skip_filename: bool,
}

impl<'a> GetFileLinkParams<'a> {
    /// Sets the `force_download` flag.
    ///
    /// # Arguments
    ///
    /// * `value` - Boolean indicating whether to force the download.
    pub fn set_force_download(&mut self, value: bool) {
        self.force_download = value;
    }

    /// Sets the `force_download` flag and returns the updated `GetFileLinkParams` object.
    ///
    /// # Arguments
    ///
    /// * `value` - Boolean indicating whether to force the download.
    pub fn with_force_download(mut self, value: bool) -> Self {
        self.set_force_download(value);
        self
    }

    /// Sets the content type for the file link.
    ///
    /// # Arguments
    ///
    /// * `value` - Content type (e.g., "application/pdf").
    pub fn set_content_type(&mut self, value: impl Into<Cow<'a, str>>) {
        self.content_type = Some(value.into());
    }

    /// Sets the content type and returns the updated `GetFileLinkParams` object.
    ///
    /// # Arguments
    ///
    /// * `value` - Content type (e.g., "application/pdf").
    pub fn with_content_type(mut self, value: impl Into<Cow<'a, str>>) -> Self {
        self.set_content_type(value);
        self
    }

    /// Sets the maximum download speed for the file.
    ///
    /// # Arguments
    ///
    /// * `value` - Maximum speed in bytes per second.
    pub fn set_max_speed(&mut self, value: u64) {
        self.max_speed = Some(value);
    }

    /// Sets the maximum download speed and returns the updated `GetFileLinkParams` object.
    ///
    /// # Arguments
    ///
    /// * `value` - Maximum speed in bytes per second.
    pub fn with_max_speed(mut self, value: u64) -> Self {
        self.set_max_speed(value);
        self
    }

    /// Sets the `skip_filename` flag.
    ///
    /// # Arguments
    ///
    /// * `value` - Boolean indicating whether to skip the filename in the URL.
    pub fn set_skip_filename(&mut self, value: bool) {
        self.skip_filename = value;
    }

    /// Sets the `skip_filename` flag and returns the updated `GetFileLinkParams` object.
    ///
    /// # Arguments
    ///
    /// * `value` - Boolean indicating whether to skip the filename in the URL.
    pub fn with_skip_filename(mut self, value: bool) -> Self {
        self.set_skip_filename(value);
        self
    }
}

/// Struct representing the parameters used when making a request to get a file link.
#[derive(serde::Serialize)]
struct Params<'a> {
    /// The identifier for the file being requested.
    #[serde(flatten)]
    identifier: FileIdentifier<'a>,

    /// The parameters controlling how the file link should be generated.
    #[serde(flatten)]
    params: GetFileLinkParams<'a>,
}

impl crate::Client {
    /// Gets a file link using only the file identifier, without additional parameters.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the file.
    ///
    /// # Returns
    ///
    /// A result containing the `StreamingLinkList` with the generated file links.
    pub async fn get_file_link(
        &self,
        identifier: impl Into<FileIdentifier<'_>>,
    ) -> crate::Result<StreamingLinkList> {
        self.get_request::<StreamingLinkList, _>("getfilelink", identifier.into())
            .await
    }

    /// Gets a file link using both the file identifier and additional parameters.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the file.
    /// * `params` - The parameters to customize the file link (e.g., force download, content type, etc.).
    ///
    /// # Returns
    ///
    /// A result containing the `StreamingLinkList` with the generated file links.
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
