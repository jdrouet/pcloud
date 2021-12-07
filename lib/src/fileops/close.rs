use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::error::Error;
use crate::request::Response;

impl BinaryClient {
    #[tracing::instrument(skip(self))]
    pub fn file_close(&mut self, fd: u64) -> Result<(), Error> {
        let params = vec![("fd", BinaryValue::Number(fd))];
        let res = self.send_command("file_close", &params)?;
        let res: Response<()> = serde_json::from_value(res)?;
        res.payload()
    }
}
