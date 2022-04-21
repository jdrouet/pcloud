use crate::binary::{BinaryClient, Value as BinaryValue};
use crate::error::Error;
use crate::prelude::BinaryCommand;
use crate::request::Response;

#[derive(Debug)]
pub struct FileCloseCommand {
    fd: u64,
}

impl FileCloseCommand {
    pub fn new(fd: u64) -> Self {
        Self { fd }
    }
}

impl BinaryCommand for FileCloseCommand {
    type Output = ();

    fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
        let params = vec![("fd", BinaryValue::Number(self.fd))];
        let res = client.send_command("file_close", &params)?;
        let res: Response<()> = serde_json::from_value(res)?;
        res.payload()
    }
}
