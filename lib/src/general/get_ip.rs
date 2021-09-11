use crate::binary::BinaryClient;
use crate::error::Error;
use crate::request::Response;

#[derive(serde::Deserialize)]
pub struct Payload {
    pub ip: String,
    pub country: String,
}

impl BinaryClient {
    pub fn get_ip(&mut self) -> Result<Payload, Error> {
        let params = Vec::new();
        let result = self.send_command("getip", &params)?;
        let result: Response<Payload> = serde_json::from_value(result)?;
        result.payload()
    }
}
