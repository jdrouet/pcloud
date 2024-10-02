use crate::file::FileIdentifier;

#[derive(Debug)]
/// Command to get video link for streaming
pub struct GetVideoLinkCommand {
    /// File identifier
    pub identifier: FileIdentifier,
    /// int audio bit rate in kilobits, from 16 to 320
    pub audio_bit_rate: Option<u16>,
    /// int video bitrate in kilobits, from 16 to 4000
    pub video_bit_rate: Option<u32>,
    /// string in pixels, from 64x64 to 1280x960, WIDTHxHEIGHT
    pub resolution: Option<String>,
    /// if set, turns off adaptive streaming and the stream will be with a constant bitrate.
    pub fixed_bit_rate: bool,
}

impl GetVideoLinkCommand {
    pub fn new(identifier: FileIdentifier) -> Self {
        Self {
            identifier,
            audio_bit_rate: None,
            video_bit_rate: None,
            resolution: None,
            fixed_bit_rate: false,
        }
    }

    pub fn set_audio_bit_rate(&mut self, value: Option<u16>) {
        self.audio_bit_rate = value;
    }

    pub fn with_audio_bit_rate(mut self, value: u16) -> Self {
        self.audio_bit_rate = Some(value);
        self
    }

    pub fn set_video_bit_rate(mut self, value: Option<u32>) {
        self.video_bit_rate = value;
    }

    pub fn with_video_bit_rate(mut self, value: u32) -> Self {
        self.video_bit_rate = Some(value);
        self
    }

    pub fn set_resolution(mut self, value: Option<String>) {
        self.resolution = value;
    }

    pub fn with_resolution(mut self, value: String) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn set_fixed_bit_rate(&mut self, value: bool) {
        self.fixed_bit_rate = value;
    }

    pub fn with_fixed_bit_rate(mut self, value: bool) -> Self {
        self.fixed_bit_rate = value;
        self
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::GetVideoLinkCommand;
    use crate::client::HttpClient;
    use crate::error::Error;
    use crate::file::FileIdentifierParam;
    use crate::prelude::HttpCommand;
    use crate::request::Response;
    use crate::streaming::Payload;

    #[derive(serde::Serialize)]
    /// Command to get video link for streaming
    struct GetVideoLinkParams {
        #[serde(flatten)]
        identifier: FileIdentifierParam,
        #[serde(rename = "abitrate", skip_serializing_if = "Option::is_none")]
        audio_bit_rate: Option<u16>,
        #[serde(rename = "vbitrate", skip_serializing_if = "Option::is_none")]
        video_bit_rate: Option<u32>,
        #[serde(skip_serializing_if = "Option::is_none")]
        resolution: Option<String>,
        #[serde(
            rename = "fixedbitrate",
            skip_serializing_if = "crate::client::is_false",
            serialize_with = "crate::client::serialize_bool"
        )]
        fixed_bit_rate: bool,
    }

    impl From<GetVideoLinkCommand> for GetVideoLinkParams {
        fn from(value: GetVideoLinkCommand) -> Self {
            Self {
                identifier: value.identifier.into(),
                audio_bit_rate: value.audio_bit_rate,
                video_bit_rate: value.video_bit_rate,
                resolution: value.resolution,
                fixed_bit_rate: value.fixed_bit_rate,
            }
        }
    }

    #[async_trait::async_trait]
    impl HttpCommand for GetVideoLinkCommand {
        type Output = String;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let params = GetVideoLinkParams::from(self);
            let result: Response<Payload> = client.get_request("getvideolink", &params).await?;
            result.payload().map(|item| item.to_url())
        }
    }
}
