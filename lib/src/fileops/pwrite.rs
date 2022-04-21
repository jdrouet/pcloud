use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::error::Error;
use crate::prelude::BinaryCommand;
use crate::request::Response;

#[derive(Debug, serde::Deserialize)]
pub struct Payload {
    bytes: usize,
}

#[derive(Debug)]
pub struct FilePWriteCommand<'d> {
    fd: u64,
    offset: usize,
    data: &'d [u8],
}

impl<'d> FilePWriteCommand<'d> {
    pub fn new(fd: u64, offset: usize, data: &'d [u8]) -> Self {
        Self { fd, offset, data }
    }

    pub fn new_chunks(fd: u64, offset: usize, data: &'d [u8]) -> Vec<Self> {
        data.chunks(super::MAX_BLOCK_SIZE)
            .enumerate()
            .map(|(index, chunk)| {
                FilePWriteCommand::new(fd, offset + index * super::MAX_BLOCK_SIZE, chunk)
            })
            .collect()
    }

    fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        vec![
            ("fd", BinaryValue::Number(self.fd)),
            ("offset", BinaryValue::Number(self.offset as u64)),
        ]
    }
}

impl<'d> BinaryCommand for FilePWriteCommand<'d> {
    type Output = usize;

    fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
        let res =
            client.send_command_with_data("file_pwrite", &self.to_binary_params(), self.data)?;
        let res: Response<Payload> = serde_json::from_value(res)?;
        res.payload().map(|value| value.bytes).map_err(|err| {
            tracing::error!("unable to read the result: {:?}", err);
            Error::ResponseFormat
        })
    }
}
