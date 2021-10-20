use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::error::Error;
use crate::request::Response;

#[derive(Debug, serde::Deserialize)]
pub struct Payload {
    bytes: usize,
}

#[derive(Debug)]
pub struct Params<'d> {
    fd: usize,
    offset: usize,
    data: &'d [u8],
}

impl<'d> Params<'d> {
    pub fn new(fd: usize, offset: usize, data: &'d [u8]) -> Self {
        Self { fd, offset, data }
    }

    fn to_binary_params(&self) -> Vec<(&str, BinaryValue)> {
        vec![
            ("fd", BinaryValue::Number(self.fd as u64)),
            ("offset", BinaryValue::Number(self.offset as u64)),
        ]
    }
}

impl BinaryClient {
    fn file_pwrite_part(&mut self, params: &Params) -> Result<usize, Error> {
        let res =
            self.send_command_with_data("file_pwrite", &params.to_binary_params(), params.data)?;
        let res: Response<Payload> = serde_json::from_value(res)?;
        res.payload().map(|value| value.bytes).map_err(|err| {
            tracing::error!("unable to read the result: {:?}", err);
            Error::ResponseFormat
        })
    }

    #[tracing::instrument(skip(self))]
    pub fn file_pwrite(&mut self, params: &Params) -> Result<usize, Error> {
        let mut offset = 0;
        for chunk in params.data.chunks(super::MAX_BLOCK_SIZE) {
            let chunk_params = Params::new(params.fd, offset, chunk);
            self.file_pwrite_part(&chunk_params)?;
            offset += chunk.len();
        }
        Ok(offset)
    }
}
