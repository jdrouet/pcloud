use crate::binary::BinaryClient;
use crate::error::Error;
use crate::prelude::BinaryCommand;
use crate::request::Response;

#[derive(Debug, Default)]
pub struct GetIpCommand;

impl BinaryCommand for GetIpCommand {
    type Output = Payload;

    fn execute(self, client: &mut BinaryClient) -> Result<Self::Output, Error> {
        let params = Vec::new();
        let result = client.send_command("getip", &params)?;
        let result: Response<Payload> = serde_json::from_value(result)?;
        result.payload()
    }
}

#[derive(serde::Deserialize)]
pub struct Payload {
    pub ip: String,
    pub country: String,
}
