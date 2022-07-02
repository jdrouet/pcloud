pub mod get_audio_link;
pub mod get_file_link;
pub mod get_video_link;

#[derive(Debug, serde::Deserialize)]
pub struct Payload {
    // expires: String,
    pub hosts: Vec<String>,
    pub path: String,
}

#[cfg(feature = "client-http")]
impl Payload {
    fn to_url(&self) -> String {
        let host = self.hosts.get(0).unwrap();
        format!("https://{}{}", host, self.path)
    }
}
