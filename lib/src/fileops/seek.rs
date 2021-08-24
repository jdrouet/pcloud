use crate::binary::{PCloudBinaryApi, Value as BinaryValue};
use crate::error::Error;
use crate::request::Response;
use std::io::SeekFrom;

#[derive(Debug, serde::Deserialize)]
struct Payload {
    offset: usize,
}

#[derive(Debug, Default)]
pub struct Params {
    fd: usize,
    offset: i64,
    whence: u8,
}

impl Params {
    pub fn new(fd: usize, pos: SeekFrom) -> Self {
        let (offset, whence): (i64, u8) = match pos {
            SeekFrom::Start(value) => (value as i64, 0),
            SeekFrom::Current(value) => (value, 1),
            SeekFrom::End(value) => (value, 2),
        };
        Self { fd, offset, whence }
    }

    fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        vec![
            ("fd", BinaryValue::Number(self.fd as u64)),
            ("offset", BinaryValue::Number(self.offset as u64)),
            ("whence", BinaryValue::Number(self.whence as u64)),
        ]
    }
}

impl PCloudBinaryApi {
    pub fn file_seek(&mut self, params: &Params) -> Result<usize, Error> {
        let params = params.to_binary_params();
        let result = self.send_command("file_seek", &params, false, 0)?;
        let result: Response<Payload> = serde_json::from_value(result)?;
        result.payload().map(|p| p.offset)
    }
}
