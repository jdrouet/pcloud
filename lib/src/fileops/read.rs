use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::error::Error;
use crate::request::Response;
use std::io::Read;

#[derive(Debug, serde::Deserialize)]
pub struct Payload {
    data: usize,
}

#[derive(Debug)]
pub struct Params {
    fd: usize,
    count: usize,
}

impl Params {
    pub fn new(fd: usize, count: usize) -> Self {
        Self { fd, count }
    }

    fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        vec![
            ("fd", BinaryValue::Number(self.fd as u64)),
            ("count", BinaryValue::Number(self.count as u64)),
        ]
    }
}

impl BinaryClient {
    #[tracing::instrument(skip(self))]
    pub fn file_read(&mut self, params: &Params) -> Result<Vec<u8>, Error> {
        eprintln!("file_read({}, {})", params.fd, params.count);
        let res = self.send_command("file_read", &params.to_binary_params())?;
        let res: Response<Payload> = serde_json::from_value(res)?;
        let length = res.payload().map(|value| value.data).map_err(|err| {
            tracing::error!("unable to read the length: {:?}", err);
            Error::ResponseFormat
        })?;
        let mut buffer: Vec<u8> = vec![0; length];
        self.stream.read_exact(&mut buffer).map_err(|err| {
            tracing::error!("unable to read the data: {:?}", err);
            Error::ResponseFormat
        })?;
        Ok(buffer)
    }
}
