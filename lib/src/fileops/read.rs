use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::error::Error;
use crate::prelude::BinaryCommand;
use crate::request::Response;
use std::io::Read;

#[derive(Debug, serde::Deserialize)]
pub struct Payload {
    data: usize,
}

#[derive(Debug)]
pub struct FileReadCommand {
    fd: u64,
    count: usize,
}

impl FileReadCommand {
    pub fn new(fd: u64, count: usize) -> Self {
        Self { fd, count }
    }

    fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        vec![
            ("fd", BinaryValue::Number(self.fd)),
            ("count", BinaryValue::Number(self.count as u64)),
        ]
    }
}

impl BinaryCommand for FileReadCommand {
    type Output = Vec<u8>;

    fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
        let res = client.send_command("file_read", &self.to_binary_params())?;
        let res: Response<Payload> = serde_json::from_value(res)?;
        let length = res.payload().map(|value| value.data).map_err(|err| {
            tracing::error!("unable to read the length: {:?}", err);
            Error::ResponseFormat
        })?;
        let mut buffer: Vec<u8> = vec![0; length];
        client.stream.read_exact(&mut buffer).map_err(|err| {
            tracing::error!("unable to read the data: {:?}", err);
            Error::ResponseFormat
        })?;
        Ok(buffer)
    }
}
