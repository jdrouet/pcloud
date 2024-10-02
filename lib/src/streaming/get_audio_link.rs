use crate::file::FileIdentifier;

#[derive(Debug)]
/// Command to get the audio link for streaming
pub struct GetAudioLinkCommand<'a> {
    /// File identifier
    pub identifier: FileIdentifier<'a>,
    /// int audio bit rate in kilobits, from 16 to 320
    pub audio_bit_rate: Option<u16>,
    /// Download with Content-Type = application/octet-stream
    pub force_download: bool,
}

impl<'a> GetAudioLinkCommand<'a> {
    pub fn new(identifier: FileIdentifier<'a>) -> Self {
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
    use crate::streaming::SteamingLinkList;

    #[derive(serde::Serialize)]
    struct GetAudioLinkParams<'a> {
        #[serde(flatten)]
        identifier: FileIdentifierParam<'a>,
        #[serde(rename = "abitrate", skip_serializing_if = "Option::is_none")]
        audio_bit_rate: Option<u16>,
        #[serde(
            rename = "forcedownload",
            skip_serializing_if = "crate::client::is_false",
            serialize_with = "crate::client::serialize_bool"
        )]
        force_download: bool,
    }

    impl<'a> From<GetAudioLinkCommand<'a>> for GetAudioLinkParams<'a> {
        fn from(value: GetAudioLinkCommand<'a>) -> Self {
            Self {
                identifier: FileIdentifierParam::from(value.identifier),
                audio_bit_rate: value.audio_bit_rate,
                force_download: value.force_download,
            }
        }
    }

    #[async_trait::async_trait]
    impl<'a> HttpCommand for GetAudioLinkCommand<'a> {
        type Output = SteamingLinkList;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            let params = GetAudioLinkParams::from(self);
            client
                .get_request::<SteamingLinkList, _>("getaudiolink", params)
                .await
        }
    }
}
