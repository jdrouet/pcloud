#[derive(Debug)]
pub enum Error {
    Payload(u16, String),
    Reqwest(reqwest::Error),
    ResponseFormat,
    Download(std::io::Error),
    Upload(std::io::Error),
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}
