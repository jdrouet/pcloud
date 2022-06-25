use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::error::Error;
use crate::prelude::BinaryCommand;
use crate::request::Response;

#[derive(Debug, serde::Deserialize)]
pub struct Payload {
    pub size: u64,
    pub offset: usize,
}

#[derive(Debug)]
pub struct FileSizeCommand {
    fd: u64,
}

impl FileSizeCommand {
    pub fn new(fd: u64) -> Self {
        Self { fd }
    }
}

impl BinaryCommand for FileSizeCommand {
    type Output = Payload;

    fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
        let params = vec![("fd", BinaryValue::Number(self.fd))];
        let res = client.send_command("file_size", &params)?;
        let res: Response<Payload> = serde_json::from_value(res)?;
        res.payload()
    }
}
