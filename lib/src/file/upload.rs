use crate::folder::FolderIdentifier;

use super::File;

/// Response returned by the `uploadfile` endpoint when uploading multiple files.
///
/// Contains the list of uploaded file IDs and their corresponding metadata.
#[derive(Debug, serde::Deserialize)]
pub struct MultipartFileUploadResponse {
    /// The IDs of the uploaded files.
    #[serde(rename = "fileids")]
    pub file_ids: Vec<u64>,

    /// Metadata for each uploaded file.
    pub metadata: Vec<File>,
}

/// Builder for uploading multiple files to pCloud.
///
/// This struct provides a convenient way to assemble a multipart form upload,
/// either from in-memory data, raw bodies, or asynchronous streams.
#[derive(Debug, Default)]
pub struct MultiFileUpload {
    parts: Vec<reqwest::multipart::Part>,
}

impl MultiFileUpload {
    /// Adds a file stream to the upload and returns the updated builder.
    ///
    /// This is a chainable version of [`MultiFileUpload::add_stream_entry`].
    ///
    /// # Arguments
    ///
    /// * `filename` - The name to assign to the uploaded file.
    /// * `length` - The size of the file in bytes.
    /// * `stream` - A `TryStream` of bytes representing the file content.
    pub fn with_stream_entry<F, S>(mut self, filename: F, length: Option<u64>, stream: S) -> Self
    where
        F: Into<String>,
        S: futures_core::stream::TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        bytes::Bytes: From<S::Ok>,
    {
        self.add_stream_entry(filename, length, stream);
        self
    }

    /// Adds a file stream to the upload.
    ///
    /// # Arguments
    ///
    /// * `filename` - The name to assign to the uploaded file.
    /// * `length` - The size of the file in bytes.
    /// * `stream` - A `TryStream` of bytes representing the file content.
    pub fn add_stream_entry<F, S>(&mut self, filename: F, length: Option<u64>, stream: S)
    where
        F: Into<String>,
        S: futures_core::stream::TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        bytes::Bytes: From<S::Ok>,
    {
        let body = reqwest::Body::wrap_stream(stream);
        self.add_body_entry(filename, length, body);
    }

    /// Adds a file from a raw body and returns the updated builder.
    ///
    /// This is a chainable version of [`MultiFileUpload::add_body_entry`].
    ///
    /// # Arguments
    ///
    /// * `filename` - The name to assign to the uploaded file.
    /// * `length` - The size of the file in bytes.
    /// * `body` - A `reqwest::Body` representing the file data.
    pub fn with_body_entry<F, B>(mut self, filename: F, length: Option<u64>, body: B) -> Self
    where
        F: Into<String>,
        B: Into<reqwest::Body>,
    {
        self.add_body_entry(filename, length, body);
        self
    }

    /// Adds a file from a raw body to the upload.
    ///
    /// # Arguments
    ///
    /// * `filename` - The name to assign to the uploaded file.
    /// * `length` - The size of the file in bytes.
    /// * `body` - A `reqwest::Body` containing the file content.
    pub fn add_body_entry<F, B>(&mut self, filename: F, length: Option<u64>, body: B)
    where
        F: Into<String>,
        B: Into<reqwest::Body>,
    {
        let part = if let Some(length) = length {
            let mut headers = reqwest::header::HeaderMap::with_capacity(1);
            let content_length = length.to_string();
            headers.append(
                reqwest::header::CONTENT_LENGTH,
                reqwest::header::HeaderValue::from_str(&content_length)
                    .expect("content-length must be a valid number"),
            );

            reqwest::multipart::Part::stream_with_length(body, length)
                .file_name(filename.into())
                .headers(headers)
        } else {
            reqwest::multipart::Part::stream(body).file_name(filename.into())
        };

        self.parts.push(part);
    }

    /// Converts the upload builder into a multipart form.
    ///
    /// This method is used internally before sending the request.
    fn into_form(self) -> reqwest::multipart::Form {
        self.parts.into_iter().enumerate().fold(
            reqwest::multipart::Form::default(),
            |form, (index, part)| form.part(format!("f{index}"), part),
        )
    }
}

impl crate::Client {
    /// Uploads multiple files to a specified folder on pCloud.
    ///
    /// This method uses multipart form submission to upload several files in a single request.
    ///
    /// # Arguments
    ///
    /// * `parent` - A value convertible into a [`FolderIdentifier`] representing the destination folder.
    /// * `files` - A [`MultiFileUpload`] builder containing the files to upload.
    ///
    /// # Returns
    ///
    /// On success, returns a list of [`File`] metadata for each uploaded file.
    ///
    /// # Errors
    ///
    /// Returns a [`crate::Error`] if the upload fails due to network issues,
    /// invalid input, or server-side errors.
    ///
    /// # Examples
    ///
    /// ```rust,ignore
    /// use bytes::Bytes;
    /// use futures_util::stream;
    ///
    /// # async fn example(client: &pcloud::Client) -> Result<(), pcloud::Error> {
    /// let data = vec![Ok(Bytes::from_static(b"hello world"))];
    /// let stream = stream::iter(data);
    ///
    /// let upload = pcloud::file::upload::MultiFileUpload::default()
    ///     .with_stream_entry("hello.txt", 11, stream);
    ///
    /// let uploaded = client.upload_files("/my-folder", upload).await?;
    /// println!("Uploaded {} file(s)", uploaded.len());
    /// # Ok(())
    /// # }
    /// ```
    pub async fn upload_files(
        &self,
        parent: impl Into<FolderIdentifier<'_>>,
        files: MultiFileUpload,
    ) -> crate::Result<Vec<File>> {
        self.post_request_multipart::<MultipartFileUploadResponse, _>(
            "uploadfile",
            parent.into(),
            files.into_form(),
        )
        .await
        .map(|res| res.metadata)
    }
}

#[cfg(test)]
mod tests {
    use crate::{file::upload::MultiFileUpload, Client, Credentials};
    use mockito::Matcher;

    #[tokio::test]
    async fn multipart_success() {
        let mut server = mockito::Server::new_async().await;
        let m_upload = server
            .mock("POST", "/uploadfile")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
            ]))
            .match_body(Matcher::Any)
            .match_header("accept", "*/*")
            .match_header("user-agent", crate::USER_AGENT)
            .match_header(
                "content-type",
                Matcher::Regex("multipart/form-data; boundary=.*".to_string()),
            )
            .match_header("content-length", Matcher::Regex("[0-9]+".to_string()))
            .with_status(200)
            .with_body(
                r#"{
        "result": 0,
        "metadata": [
            {
                "name": "big-file.bin",
                "created": "Tue, 09 Aug 2022 13:43:17 +0000",
                "thumb": false,
                "modified": "Tue, 09 Aug 2022 13:43:17 +0000",
                "isfolder": false,
                "fileid": 15669308155,
                "hash": 15418918230810325691,
                "path": "/big-file.bin",
                "category": 0,
                "id": "f15669308155",
                "isshared": false,
                "ismine": true,
                "size": 1073741824,
                "parentfolderid": 0,
                "contenttype": "application/octet-stream",
                "icon": "file"
            }
        ],
        "checksums": [
            {
                "sha1": "a91d3c45d2ff6dc99ed3d1c150f6fae91b2e10a1",
                "sha256": "2722eb2ec44a8f5655df8ef3b7c6a1658de40d5aedcab26b3e6d043222681360"
            }
        ],
        "fileids": [15669308155]
    }"#,
            )
            .create();
        let client = Client::new(server.url(), Credentials::access_token("access-token")).unwrap();
        //
        let file = tokio::fs::File::open("./readme.md").await.unwrap();
        let length = std::fs::metadata("./readme.md").unwrap().len();
        let files = MultiFileUpload::default().with_body_entry("big-file.bin", Some(length), file);
        let result = client.upload_files(0, files).await.unwrap();
        //
        assert_eq!(result.len(), 1);
        m_upload.assert();
    }
}
