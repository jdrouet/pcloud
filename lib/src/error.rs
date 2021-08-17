#[derive(Debug)]
pub enum Error {
    Payload(u16, String),
    Reqwest(reqwest::Error),
    ResponseFormat,
    Download(std::io::Error),
    Upload(std::io::Error),
}

impl std::fmt::Display for Error {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "{:?}", self)
    }
}

impl From<reqwest::Error> for Error {
    fn from(err: reqwest::Error) -> Self {
        Self::Reqwest(err)
    }
}
