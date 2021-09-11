use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::error::Error;
use crate::request::Response;

impl BinaryClient {
    pub fn file_close(&mut self, fd: usize) -> Result<(), Error> {
        let params = vec![("fd", BinaryValue::Number(fd as u64))];
        let res = self.send_command("file_close", &params)?;
        let res: Response<()> = serde_json::from_value(res)?;
        res.payload()
    }
}
