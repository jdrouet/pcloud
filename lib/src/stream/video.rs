use std::borrow::Cow;

use super::StreamingLinkList;
use crate::file::FileIdentifier;

/// Parameters for retrieving a video file link, including options for controlling the audio bit rate,
/// video bit rate, resolution, and fixed bitrate streaming.
#[derive(Debug, Default, serde::Serialize)]
pub struct GetVideoLinkParams<'a> {
    /// Audio bit rate in kilobits per second (from 16 to 320).
    #[serde(rename = "abitrate", skip_serializing_if = "Option::is_none")]
    pub audio_bit_rate: Option<u16>,

    /// Video bit rate in kilobits per second (from 16 to 4000).
    #[serde(rename = "vbitrate", skip_serializing_if = "Option::is_none")]
    pub video_bit_rate: Option<u32>,

    /// Resolution of the video in the format WIDTHxHEIGHT (e.g., 1280x960), with a range of 64x64 to 1280x960.
    #[serde(skip_serializing_if = "Option::is_none")]
    pub resolution: Option<Cow<'a, str>>,

    /// If set to true, disables adaptive streaming and forces the video stream to have a constant bitrate.
    #[serde(
        rename = "fixedbitrate",
        skip_serializing_if = "crate::request::is_false",
        serialize_with = "crate::request::serialize_bool"
    )]
    pub fixed_bit_rate: bool,
}

impl<'a> GetVideoLinkParams<'a> {
    /// Sets the audio bit rate for the video link.
    ///
    /// # Arguments
    ///
    /// * `value` - Audio bit rate in kilobits per second (from 16 to 320).
    pub fn set_audio_bit_rate(&mut self, value: u16) {
        self.audio_bit_rate = Some(value);
    }

    /// Sets the audio bit rate and returns the updated `GetVideoLinkParams` object.
    ///
    /// # Arguments
    ///
    /// * `value` - Audio bit rate in kilobits per second (from 16 to 320).
    pub fn with_audio_bit_rate(mut self, value: u16) -> Self {
        self.set_audio_bit_rate(value);
        self
    }

    /// Sets the video bit rate for the video link.
    ///
    /// # Arguments
    ///
    /// * `value` - Video bit rate in kilobits per second (from 16 to 4000).
    pub fn set_video_bit_rate(&mut self, value: u32) {
        self.video_bit_rate = Some(value);
    }

    /// Sets the video bit rate and returns the updated `GetVideoLinkParams` object.
    ///
    /// # Arguments
    ///
    /// * `value` - Video bit rate in kilobits per second (from 16 to 4000).
    pub fn with_video_bit_rate(mut self, value: u32) -> Self {
        self.set_video_bit_rate(value);
        self
    }

    /// Sets the resolution for the video link.
    ///
    /// # Arguments
    ///
    /// * `value` - The resolution in the format WIDTHxHEIGHT (e.g., 1280x960).
    pub fn set_resolution(&mut self, value: impl Into<Cow<'a, str>>) {
        self.resolution = Some(value.into());
    }

    /// Sets the resolution and returns the updated `GetVideoLinkParams` object.
    ///
    /// # Arguments
    ///
    /// * `value` - The resolution in the format WIDTHxHEIGHT (e.g., 1280x960).
    pub fn with_resolution(mut self, value: impl Into<Cow<'a, str>>) -> Self {
        self.set_resolution(value);
        self
    }

    /// Sets the `fixed_bit_rate` flag.
    ///
    /// # Arguments
    ///
    /// * `value` - A boolean indicating whether the video should have a fixed bitrate.
    pub fn set_fixed_bit_rate(&mut self, value: bool) {
        self.fixed_bit_rate = value;
    }

    /// Sets the `fixed_bit_rate` flag and returns the updated `GetVideoLinkParams` object.
    ///
    /// # Arguments
    ///
    /// * `value` - A boolean indicating whether the video should have a fixed bitrate.
    pub fn with_fixed_bit_rate(mut self, value: bool) -> Self {
        self.set_fixed_bit_rate(value);
        self
    }
}

/// Struct representing the parameters used when making a request to get a video link.
#[derive(serde::Serialize)]
struct Params<'a> {
    /// The identifier for the file being requested.
    #[serde(flatten)]
    identifier: FileIdentifier<'a>,

    /// The parameters controlling how the video link should be generated.
    #[serde(flatten)]
    params: GetVideoLinkParams<'a>,
}

impl crate::Client {
    /// Gets a video link using only the file identifier, without additional parameters.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the file.
    ///
    /// # Returns
    ///
    /// A result containing the `StreamingLinkList` with the generated video links.
    pub async fn get_video_link(
        &self,
        identifier: impl Into<FileIdentifier<'_>>,
    ) -> crate::Result<StreamingLinkList> {
        self.get_request::<StreamingLinkList, _>("getvideolink", identifier.into())
            .await
    }

    /// Gets a video link using both the file identifier and additional parameters.
    ///
    /// # Arguments
    ///
    /// * `identifier` - The identifier of the file.
    /// * `params` - The parameters to customize the video link (e.g., audio bit rate, video bit rate, resolution).
    ///
    /// # Returns
    ///
    /// A result containing the `StreamingLinkList` with the generated video links.
    pub async fn get_video_link_with_params(
        &self,
        identifier: impl Into<FileIdentifier<'_>>,
        params: GetVideoLinkParams<'_>,
    ) -> crate::Result<StreamingLinkList> {
        self.get_request::<StreamingLinkList, _>(
            "getvideolink",
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
        let m = server.mock("GET", "/getvideolink")
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
        let result = client.get_video_link(42).await.unwrap();
        let mut iter = result.links();
        assert_eq!(iter.next().unwrap().to_string(), "https://edef2.pcloud.com/DLZCAt2vXZejNfL5ZruLVZZTk2ev7Z2ZZNR5ZZdoz6ZXZQZZErw4bH0PfzBQt3LlgXMliXVtietX/SAkdyBjkA7mQABbT.bin");
        m.assert();
    }
}
