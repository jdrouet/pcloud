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
    // int video bitrate in kilobits, from 16 to 4000
    video_bit_rate: Option<u32>,
    // string in pixels, from 64x64 to 1280x960, WIDTHxHEIGHT
    resolution: Option<String>,
    // if set, turns off adaptive streaming and the stream will be with a constant bitrate.
    fixed_bit_rate: bool,
}

impl Params {
    pub fn new<I: Into<FileIdentifier>>(identifier: I) -> Self {
        Self {
            identifier: identifier.into(),
            audio_bit_rate: None,
            video_bit_rate: None,
            resolution: None,
            fixed_bit_rate: false,
        }
    }

    pub fn audio_bit_rate(mut self, value: u16) -> Self {
        self.audio_bit_rate = Some(value);
        self
    }

    pub fn video_bit_rate(mut self, value: u32) -> Self {
        self.video_bit_rate = Some(value);
        self
    }

    pub fn resolution(mut self, value: String) -> Self {
        self.resolution = Some(value);
        self
    }

    pub fn fixed_bit_rate(mut self, value: bool) -> Self {
        self.fixed_bit_rate = value;
        self
    }

    pub fn to_http_params(&self) -> Vec<(&str, String)> {
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

impl PCloudHttpApi {
    pub async fn get_video_link(&self, params: &Params) -> Result<String, Error> {
        let result: Response<Payload> = self
            .get_request("getvideolink", &params.to_http_params())
            .await?;
        result.payload().map(|value| value.to_url())
    }
}
