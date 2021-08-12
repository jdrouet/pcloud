use crate::error::Error;

pub const ROOT_FOLDER: usize = 0;

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
pub enum Response<T> {
    Error {
        result: u16,
        error: String,
    },
    Success {
        result: u16,
        #[serde(flatten)]
        payload: T,
    },
}

impl<T> Response<T> {
    pub fn payload(self) -> Result<T, Error> {
        match self {
            Self::Error { result, error } => Err(Error::Payload(result, error)),
            Self::Success { payload, .. } => Ok(payload),
        }
    }
}
