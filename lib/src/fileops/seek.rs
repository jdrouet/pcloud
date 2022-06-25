use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::error::Error;
use crate::prelude::BinaryCommand;
use crate::request::Response;
use std::io::SeekFrom;

#[derive(Debug, serde::Deserialize)]
struct Payload {
    offset: usize,
}

#[derive(Debug, Default)]
pub struct FileSeekCommand {
    fd: u64,
    offset: i64,
    whence: u8,
}

impl FileSeekCommand {
    pub fn new(fd: u64, pos: SeekFrom) -> Self {
        let (offset, whence): (i64, u8) = match pos {
            SeekFrom::Start(value) => (value as i64, 0),
            SeekFrom::Current(value) => (value, 1),
            SeekFrom::End(value) => (value, 2),
        };
        Self { fd, offset, whence }
    }

    fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        vec![
            ("fd", BinaryValue::Number(self.fd)),
            ("offset", BinaryValue::Number(self.offset as u64)),
            ("whence", BinaryValue::Number(self.whence as u64)),
        ]
    }
}

impl BinaryCommand for FileSeekCommand {
    type Output = usize;

    fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
        let result = client.send_command("file_seek", &self.to_binary_params())?;
        let result: Response<Payload> = serde_json::from_value(result)?;
        result.payload().map(|p| p.offset)
    }
}
