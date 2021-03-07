use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::error::Error;
use crate::request::Response;

#[derive(Debug, serde::Deserialize)]
pub struct Payload {
    pub size: u64,
    pub offset: usize,
}

impl BinaryClient {
    #[tracing::instrument(skip(self))]
    pub fn file_size(&mut self, fd: u64) -> Result<Payload, Error> {
        let params = vec![("fd", BinaryValue::Number(fd))];
        let res = self.send_command("file_size", &params)?;
        let res: Response<Payload> = serde_json::from_value(res)?;
        res.payload()
    }
}
