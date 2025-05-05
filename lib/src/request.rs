//! The client implementing the [HTTP Json protocol](https://docs.pcloud.com/protocols/http_json_protocol/)

use crate::{Credentials, Error};

/// Reads the HTTP response and attempts to deserialize it into a type `T`.
/// If the response is successful, it returns the payload, otherwise, it returns an error.
async fn read_response<T: serde::de::DeserializeOwned>(res: reqwest::Response) -> Result<T, Error> {
    let status = res.status();
    tracing::debug!("responded with status {status:?}");
    res.json::<Response<T>>()
        .await
        .map_err(Error::from)
        .and_then(Response::payload)
}

impl crate::Client {
    /// Constructs a URL for a specific method.
    ///
    /// # Arguments
    ///
    /// * `method` - The method or endpoint to be appended to the base URL.
    ///
    /// # Returns
    ///
    /// A formatted string containing the full URL.
    fn build_url(&self, method: &str) -> String {
        format!("{}/{}", self.base_url, method)
    }

    /// Sends a GET request with query parameters and deserializes the response into type `T`.
    ///
    /// # Arguments
    ///
    /// * `method` - The method or endpoint to be used in the request.
    /// * `params` - The parameters to be sent with the GET request.
    ///
    /// # Returns
    ///
    /// A result containing the deserialized response payload or an error.
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

    /// Sends a PUT request with query parameters and a binary payload, and deserializes the response into type `T`.
    ///
    /// # Arguments
    ///
    /// * `method` - The method or endpoint to be used in the request.
    /// * `params` - The parameters to be sent with the PUT request.
    /// * `payload` - The binary payload to be included in the body of the PUT request.
    ///
    /// # Returns
    ///
    /// A result containing the deserialized response payload or an error.
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

    /// Sends a POST request with multipart form data and query parameters, and deserializes the response into type `T`.
    ///
    /// # Arguments
    ///
    /// * `method` - The method or endpoint to be used in the request.
    /// * `params` - The parameters to be sent with the POST request.
    /// * `form` - The multipart form data to be included in the body of the POST request.
    ///
    /// # Returns
    ///
    /// A result containing the deserialized response payload or an error.
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

/// Struct for serializing credentials along with the request parameters.
///
/// # Type Parameters
/// * `I` - The inner type that holds the parameters of the request.
#[derive(serde::Serialize)]
struct WithCredentials<'a, I> {
    /// The credentials for the request.
    #[serde(flatten)]
    credentials: &'a Credentials,

    /// The parameters to be sent with the request.
    #[serde(flatten)]
    inner: I,
}

/// A utility function that returns `true` if the value is `false`.
/// Used for serializing boolean values as `0` (false) or `1` (true).
///
/// # Arguments
/// * `value` - A boolean value to check.
///
/// # Returns
/// A boolean value indicating if the provided value is false.
pub(crate) fn is_false(value: &bool) -> bool {
    !*value
}

/// A custom serializer for boolean values that serializes them as strings `"0"` or `"1"`.
///
/// # Arguments
/// * `value` - The boolean value to serialize.
/// * `serializer` - The serializer to use for serialization.
///
/// # Returns
/// A result indicating the success or failure of serialization.
pub(crate) fn serialize_bool<S>(value: &bool, serializer: S) -> Result<S::Ok, S::Error>
where
    S: serde::Serializer,
{
    serializer.serialize_str(if *value { "1" } else { "0" })
}

/// Enum representing the HTTP response from the server, which can either be an error or a success.
///
/// # Type Parameters
/// * `T` - The type of the response payload in case of success.
#[derive(Debug, serde::Deserialize)]
#[serde(untagged)]
enum Response<T> {
    /// Represents an error response from the server.
    Error { result: u16, error: String },

    /// Represents a successful response from the server.
    Success {
        #[allow(unused)]
        result: u16,
        #[serde(flatten)]
        payload: T,
    },
}

impl<T> Response<T> {
    /// Extracts the payload from the response.
    ///
    /// If the response is a success, it returns the payload, otherwise, it returns an error.
    ///
    /// # Returns
    /// A result containing the payload if the response is a success, or an error if the response is an error.
    fn payload(self) -> Result<T, Error> {
        match self {
            Self::Error { result, error } => Err(Error::Protocol(result, error)),
            Self::Success { payload, .. } => Ok(payload),
        }
    }
}
