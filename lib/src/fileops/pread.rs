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
    offset: usize,
}

impl Params {
    pub fn new(fd: usize, count: usize, offset: usize) -> Self {
        Self { fd, count, offset }
    }

    fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        vec![
            ("fd", BinaryValue::Number(self.fd as u64)),
            (
                "count",
                BinaryValue::Number(super::MAX_BLOCK_SIZE.min(self.count) as u64),
            ),
            ("offset", BinaryValue::Number(self.offset as u64)),
        ]
    }
}

impl BinaryClient {
    fn file_pread_part(&mut self, params: &Params) -> Result<Vec<u8>, Error> {
        let res = self.send_command("file_pread", &params.to_binary_params())?;
        let res: Response<Payload> = serde_json::from_value(res)?;
        let length = res.payload().map(|value| value.data).map_err(|err| {
            tracing::error!("unable to read the length: {:?}", err);
            Error::ResponseFormat
        })?;
        if length == 0 {
            return Ok(Vec::new());
        }
        let mut buffer: Vec<u8> = vec![0; length];
        self.stream.read_exact(&mut buffer).map_err(|err| {
            tracing::error!("unable to read the data: {:?}", err);
            Error::ResponseFormat
        })?;
        Ok(buffer)
    }

    #[tracing::instrument(skip(self))]
    pub fn file_pread(&mut self, params: &Params) -> Result<Vec<u8>, Error> {
        let mut buffer = Vec::new();
        while buffer.len() < params.count {
            let count = params.count.min(super::MAX_BLOCK_SIZE);
            let part_params = Params::new(params.fd, count, params.offset + buffer.len());
            let part = self.file_pread_part(&part_params)?;
            buffer.extend(part);
        }
        Ok(buffer)
    }
}
