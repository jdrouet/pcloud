use crate::request::{Error, Response};
use crate::PCloudApi;

#[derive(Debug, serde::Deserialize)]
struct Payload {
    hosts: Vec<String>,
    path: String,
}

impl Payload {
    fn to_url(&self) -> String {
        let host = self.hosts.get(0).unwrap();
        format!("https://{}{}", host, self.path)
    }
}

impl PCloudApi {
    pub async fn get_link_file(&self, file_id: usize) -> Result<String, Error> {
        let file_id = file_id.to_string();
        let params = vec![("fileid", file_id.as_str())];
        let result: Response<Payload> = self.get_request("getfilelink", &params).await?;
        result.payload().map(|res| res.to_url())
    }
}
