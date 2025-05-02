//! The client implementing the [HTTP Json protocol](https://docs.pcloud.com/protocols/http_json_protocol/)

use crate::{Credentials, Error};

/// The default user agent for the http client
pub const USER_AGENT: &str = concat!(env!("CARGO_PKG_NAME"), "/", env!("CARGO_PKG_VERSION"));
/// The default part size when uploading files
pub const DEFAULT_PART_SIZE: usize = 10485760;

async fn read_response<T: serde::de::DeserializeOwned>(res: reqwest::Response) -> Result<T, Error> {
    let status = res.status();
    tracing::debug!("responded with status {status:?}");
    res.json::<Response<T>>()
        .await
        .map_err(Error::from)
        .and_then(Response::payload)
}

impl crate::Client {
    fn build_url(&self, method: &str) -> String {
        format!("{}/{}", self.base_url, method)
    }

    #[tracing::instrument(name = "get", skip(self, params))]
    pub(crate) async fn get_request<T: serde::de::DeserializeOwned, P: serde::Serialize>(
        &self,
        method: &str,
        params: P,
    ) -> Result<T, Error> {
        let uri = self.build_url(method);
        let res = self
            .inner
            .get(uri)
            .query(&WithCredentials {
                credentials: &self.credentials,
                inner: params,
            })
            .send()
            .await?;
        read_response(res).await
    }

    #[tracing::instrument(name = "put", skip(self, params))]
    pub(crate) async fn put_request_data<T: serde::de::DeserializeOwned, P: serde::Serialize>(
        &self,
        method: &str,
        params: P,
        payload: Vec<u8>,
    ) -> Result<T, Error> {
        let uri = self.build_url(method);
        let res = self
            .inner
            .put(uri)
            .query(&WithCredentials {
                credentials: &self.credentials,
                inner: params,
            })
            .body(payload)
            .send()
            .await?;
        read_response(res).await
    }

    #[tracing::instrument(name = "post", skip(self, params))]
    pub(crate) async fn post_request_multipart<
        T: serde::de::DeserializeOwned,
        P: serde::Serialize,
    >(
        &self,
        method: &str,
        params: P,
        form: reqwest::multipart::Form,
    ) -> Result<T, Error> {
        let uri = self.build_url(method);
        let res = self
            .inner
            .post(uri)
            .query(&WithCredentials {
                credentials: &self.credentials,
                inner: params,
            })
            .multipart(form)
            .send()
            .await?;
        read_response(res).await
    }
}

#[derive(serde::Serialize)]
struct WithCredentials<'a, I> {
    #[serde(flatten)]
    credentials: &'a Credentials,
    #[serde(flatten)]
    inner: I,
}

pub(crate) fn is_false(value: &bool) -> bool {
    !*value
}

pub(crate) fn serialize_bool<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(if *value { "1" } else { "0" })
}

#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum Response<T> {
    Error {
        result: u16,
        error: String,
    },
    Success {
        #[allow(unused)]
        result: u16,
        #[serde(flatten)]
        payload: T,
    },
}

impl<T> Response<T> {
    fn payload(self) -> Result<T, Error> {
        match self {
            Self::Error { result, error } => Err(Error::Protocol(result, error)),
            Self::Success { payload, .. } => Ok(payload),
        }
    }
}
