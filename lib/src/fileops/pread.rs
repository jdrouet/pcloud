use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::error::Error;
use crate::prelude::BinaryCommand;
use crate::request::Response;
use std::io::Read;

struct RangeIterator {
    count: usize,
    offset: usize,
}

impl RangeIterator {
    fn new(count: usize, offset: usize) -> Self {
        Self { count, offset }
    }
}

impl Iterator for RangeIterator {
    type Item = (usize, usize); // (count, offset)
    fn next(&mut self) -> Option<Self::Item> {
        if self.count > 0 {
            let count = super::MAX_BLOCK_SIZE.min(self.count);
            let offset = self.offset;
            self.count -= count;
            self.offset += count;
            Some((count, offset))
        } else {
            None
        }
    }
}

#[derive(Debug, serde::Deserialize)]
pub struct Payload {
    data: usize,
}

#[derive(Debug)]
pub struct FilePReadCommand {
    fd: u64,
    count: usize,
    offset: usize,
}

impl FilePReadCommand {
    pub fn new(fd: u64, count: usize, offset: usize) -> Self {
        Self { fd, count, offset }
    }

    pub fn new_chunks(fd: u64, count: usize, offset: usize) -> Vec<Self> {
        RangeIterator::new(count, offset)
            .map(|(count, offset)| FilePReadCommand::new(fd, count, offset))
            .collect()
    }

    fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        vec![
            ("fd", BinaryValue::Number(self.fd)),
            ("count", BinaryValue::Number(self.count as u64)),
            ("offset", BinaryValue::Number(self.offset as u64)),
        ]
    }
}

impl BinaryCommand for FilePReadCommand {
    type Output = Vec<u8>;

    fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
        let res = client.send_command("file_pread", &self.to_binary_params())?;
        let res: Response<Payload> = serde_json::from_value(res)?;
        let length = res.payload().map(|value| value.data).map_err(|err| {
            tracing::error!("unable to read the length: {:?}", err);
            Error::ResponseFormat
        })?;
        if length == 0 {
            return Ok(Vec::new());
        }
        let mut buffer: Vec<u8> = vec![0; length];
        client.stream.read_exact(&mut buffer).map_err(|err| {
            tracing::error!("unable to read the data: {:?}", err);
            Error::ResponseFormat
        })?;
        Ok(buffer)
    }
}
