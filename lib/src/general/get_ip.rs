#[derive(Debug, Default)]
pub struct GetIpCommand;

#[derive(serde::Deserialize)]
pub struct Payload {
    pub ip: String,
    pub country: String,
}
