use crate::file::FileIdentifier;

#[derive(Debug)]
/// Command to get the audio link for streaming
pub struct GetAudioLinkCommand {
    /// File identifier
    pub identifier: FileIdentifier,
    /// int audio bit rate in kilobits, from 16 to 320
    pub audio_bit_rate: Option<u16>,
    /// Download with Content-Type = application/octet-stream
    pub force_download: bool,
}

impl GetAudioLinkCommand {
    pub fn new(identifier: FileIdentifier) -> Self {
        Self {
            identifier,
            audio_bit_rate: None,
            force_download: false,
        }
    }

    pub fn set_audio_bit_rate(&mut self, value: Option<u16>) {
        self.audio_bit_rate = value;
    }

    pub fn with_audio_bit_rate(mut self, value: u16) -> Self {
        self.audio_bit_rate = Some(value);
        self
    }

    pub fn set_force_download(&mut self, value: bool) {
        self.force_download = value;
    }

    pub fn with_force_download(mut self, value: bool) -> Self {
        self.force_download = value;
        self
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::GetAudioLinkCommand;
    use crate::client::HttpClient;
    use crate::error::Error;
    use crate::file::FileIdentifierParam;
    use crate::prelude::HttpCommand;
    use crate::request::Response;
    use crate::streaming::Payload;

    #[derive(serde::Serialize)]
    struct GetAudioLinkParams {
        #[serde(flatten)]
        identifier: FileIdentifierParam,
        #[serde(rename = "abitrate", skip_serializing_if = "Option::is_none")]
        audio_bit_rate: Option<u16>,
        #[serde(
            rename = "forcedownload",
            skip_serializing_if = "crate::client::is_false",
            serialize_with = "crate::client::serialize_bool"
        )]
        force_download: bool,
    }

    impl From<GetAudioLinkCommand> for GetAudioLinkParams {
        fn from(value: GetAudioLinkCommand) -> Self {
            Self {
                identifier: FileIdentifierParam::from(value.identifier),
                audio_bit_rate: value.audio_bit_rate,
                force_download: value.force_download,
            }
        }
    }

    #[async_trait::async_trait]
    impl HttpCommand for GetAudioLinkCommand {
        type Output = String;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let params = GetAudioLinkParams::from(self);
            let result: Response<Payload> = client.get_request("getaudiolink", &params).await?;
            result.payload().map(|item| item.to_url())
        }
    }
}
