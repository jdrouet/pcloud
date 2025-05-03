use crate::folder::FolderIdentifier;

use super::File;

#[derive(Debug, serde::Deserialize)]
pub struct MultipartFileUploadResponse {
    #[serde(rename = "fileids")]
    pub file_ids: Vec<u64>,
    pub metadata: Vec<File>,
}

#[derive(Debug, Default)]
pub struct MultiFileUpload {
    parts: Vec<reqwest::multipart::Part>,
}

impl MultiFileUpload {
    pub fn with_stream_entry<F, S>(mut self, filename: F, length: u64, stream: S) -> Self
    where
        F: Into<String>,
        S: futures_core::stream::TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        bytes::Bytes: From<S::Ok>,
    {
        self.add_stream_entry(filename, length, stream);
        self
    }

    pub fn add_stream_entry<F, S>(&mut self, filename: F, length: u64, stream: S)
    where
        F: Into<String>,
        S: futures_core::stream::TryStream + Send + Sync + 'static,
        S::Error: Into<Box<dyn std::error::Error + Send + Sync>>,
        bytes::Bytes: From<S::Ok>,
    {
        let body = reqwest::Body::wrap_stream(stream);
        self.add_body_entry(filename, length, body);
    }

    pub fn with_body_entry<F, B>(mut self, filename: F, length: u64, body: B) -> Self
    where
        F: Into<String>,
        B: Into<reqwest::Body>,
    {
        self.add_body_entry(filename, length, body);
        self
    }

    pub fn add_body_entry<F, B>(&mut self, filename: F, length: u64, body: B)
    where
        F: Into<String>,
        B: Into<reqwest::Body>,
    {
        let mut headers = reqwest::header::HeaderMap::with_capacity(1);
        let content_length = length.to_string();
        headers.append(
            reqwest::header::CONTENT_LENGTH,
            reqwest::header::HeaderValue::from_str(content_length.as_str())
                .expect("content-length to be a valid number"),
        );
        let part = reqwest::multipart::Part::stream_with_length(body, length)
            .file_name(filename.into())
            .headers(headers);

        self.parts.push(part);
    }

    fn into_form(self) -> reqwest::multipart::Form {
        self.parts.into_iter().enumerate().fold(
            reqwest::multipart::Form::default(),
            |form, (index, part)| form.part(format!("f{index}"), part),
        )
    }
}

impl crate::Client {
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
        let files = MultiFileUpload::default().with_body_entry("big-file.bin", length, file);
        let result = client.upload_files(0, files).await.unwrap();
        //
        assert_eq!(result.len(), 1);
        m_upload.assert();
    }
}
