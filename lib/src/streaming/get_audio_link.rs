use super::Payload;
use crate::error::Error;
use crate::file::FileIdentifier;
use crate::http::PCloudHttpApi;
use crate::request::Response;

#[derive(Debug)]
pub struct Params {
    identifier: FileIdentifier,
    // int audio bit rate in kilobits, from 16 to 320
    audio_bit_rate: Option<u16>,
    // Download with Content-Type = application/octet-stream
    force_download: bool,
}

impl Params {
    pub fn new<I: Into<FileIdentifier>>(identifier: I) -> Self {
        Self {
            identifier: identifier.into(),
            audio_bit_rate: None,
            force_download: false,
        }
    }

    pub fn audio_bit_rate(mut self, value: u16) -> Self {
        self.audio_bit_rate = Some(value);
        self
    }

    pub fn force_download(mut self, value: bool) -> Self {
        self.force_download = value;
        self
    }

    pub fn to_http_params(&self) -> Vec<(&str, String)> {
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

impl PCloudHttpApi {
    pub async fn get_audio_link(&self, params: &Params) -> Result<String, Error> {
        let result: Response<Payload> = self
            .get_request("getaudiolink", &params.to_http_params())
            .await?;
        result.payload().map(|value| value.to_url())
    }
}
