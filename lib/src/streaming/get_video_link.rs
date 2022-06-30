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

    pub fn audio_bit_rate(mut self, value: u16) -> Self {
        self.audio_bit_rate = Some(value);
        self
    }

    pub fn set_video_bit_rate(mut self, value: Option<u32>) {
        self.video_bit_rate = value;
    }

    pub fn video_bit_rate(mut self, value: u32) -> Self {
        self.video_bit_rate = Some(value);
        self
    }

    pub fn set_resolution(mut self, value: Option<String>) {
        self.resolution = value;
    }

    pub fn resolution(mut self, value: String) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn set_fixed_bit_rate(&mut self, value: bool) {
        self.fixed_bit_rate = value;
    }

    pub fn fixed_bit_rate(mut self, value: bool) -> Self {
        self.fixed_bit_rate = value;
        self
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::GetVideoLinkCommand;
    use crate::error::Error;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::request::Response;
    use crate::streaming::Payload;

    impl GetVideoLinkCommand {
        fn to_http_params(&self) -> Vec<(&str, String)> {
            let mut res = self.identifier.to_http_params();
            if let Some(abitrate) = self.audio_bit_rate {
                res.push(("abitrate", abitrate.to_string()));
            }
            if let Some(vbitrate) = self.video_bit_rate {
                res.push(("vbitrate", vbitrate.to_string()));
            }
            if let Some(ref resolution) = self.resolution {
                res.push(("resolution", resolution.to_string()));
            }
            if self.fixed_bit_rate {
                res.push(("fixedbitrate", 1.to_string()));
            }
            res
        }
    }

    #[async_trait::async_trait(?Send)]
    impl HttpCommand for GetVideoLinkCommand {
        type Output = String;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let result: Response<Payload> = client
                .get_request("getvideolink", &self.to_http_params())
                .await?;
            result.payload().map(|item| item.to_url())
        }
    }
}
