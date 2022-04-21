use super::Payload;
use crate::error::Error;
use crate::file::FileIdentifier;
use crate::http::HttpClient;
use crate::prelude::Command;
use crate::request::Response;

#[derive(Debug)]
pub struct GetAudioLinkCommand {
    identifier: FileIdentifier,
    // int audio bit rate in kilobits, from 16 to 320
    audio_bit_rate: Option<u16>,
    // Download with Content-Type = application/octet-stream
    force_download: bool,
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

    pub fn audio_bit_rate(mut self, value: u16) -> Self {
        self.audio_bit_rate = Some(value);
        self
    }

    pub fn set_force_download(&mut self, value: bool) {
        self.force_download = value;
    }

    pub fn force_download(mut self, value: bool) -> Self {
        self.force_download = value;
        self
    }

    fn to_http_params(&self) -> Vec<(&str, String)> {
        let mut res = self.identifier.to_http_params();
        if let Some(abitrate) = self.audio_bit_rate {
            res.push(("abitrate", abitrate.to_string()));
        }
        if self.force_download {
            res.push(("forcedownload", 1.to_string()));
        }
        res
    }
}

#[async_trait::async_trait(?Send)]
impl Command for GetAudioLinkCommand {
    type Output = String;
    type Error = Error;

    async fn execute(self, client: &HttpClient) -> Result<Self::Output, Self::Error> {
        let result: Response<Payload> = client
            .get_request("getaudiolink", &self.to_http_params())
            .await?;
        result.payload().map(|item| item.to_url())
    }
}
