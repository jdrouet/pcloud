use crate::binary::{PCloudBinaryApi, Value as BinaryValue};
use crate::error::Error;
use crate::request::Response;

#[derive(Debug, serde::Deserialize)]
pub struct Payload {
    pub size: usize,
    pub offset: usize,
}

impl PCloudBinaryApi {
    pub fn file_size(&mut self, fd: usize) -> Result<Payload, Error> {
        let params = vec![("fd", BinaryValue::Number(fd as u64))];
        let res = self.send_command("file_size", &params, false, 0)?;
        let res: Response<Payload> = serde_json::from_value(res)?;
        res.payload()
    }
}
