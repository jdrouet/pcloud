//! Resources needed to upload a file

use std::{borrow::Cow, io::Read};

/// Default size for splitting into chunks
pub const DEFAULT_PART_SIZE: usize = 10485760;

#[derive(Debug)]
#[cfg(feature = "client-http")]
pub struct MultipartFileUploadCommand {
    pub entries: Vec<reqwest::multipart::Part>,
    pub folder_id: u64,
}

#[cfg(feature = "client-http")]
impl MultipartFileUploadCommand {
    pub fn new(folder_id: u64) -> Self {
        Self {
            entries: Vec::new(),
            folder_id,
        }
    }

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
        self.entries.push(part);
    }
}

/// Command to upload one or multiple files to a defined folder and name
///
/// Executing this command will return a `Vec` of [`File`](crate::entry::File) on success.
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/file/uploadfile.html).
///
/// # Example using the [`HttpClient`](crate::client::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::client::HttpClientBuilder;
/// use pcloud::prelude::HttpCommand;
/// use pcloud::file::upload::MultipartFileUploadCommand;
/// use tokio::fs::File;
///
/// # tokio_test::block_on(async {
/// let fname = "Cargo.toml";
/// let fsize = std::fs::metadata(fname).unwrap().len();
/// let file = File::open(fname).await.unwrap();
/// let client = HttpClientBuilder::from_env().build().unwrap();
/// let cmd = MultipartFileUploadCommand::new(12)
///     .with_body_entry(fname, fsize, file);
/// match cmd.execute(&client).await {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// # })
/// ```
#[derive(Debug, serde::Deserialize)]
pub struct MultipartFileUploadResponse {
    #[serde(rename = "fileids")]
    pub file_ids: Vec<u64>,
    pub metadata: Vec<crate::entry::File>,
}

/// Command to upload a file to a defined folder and name
///
/// Executing this command will return a [`File`](crate::entry::File) on success.
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/file/uploadfile.html).
///
/// # Example using the [`HttpClient`](crate::client::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::client::HttpClientBuilder;
/// use pcloud::prelude::HttpCommand;
/// use pcloud::file::upload::FileUploadCommand;
/// use std::fs::File;
///
/// # tokio_test::block_on(async {
/// let file = File::open("Cargo.toml").unwrap();
/// let client = HttpClientBuilder::from_env().build().unwrap();
/// let cmd = FileUploadCommand::new("Cargo.toml", 12, file);
/// match cmd.execute(&client).await {
///   Ok(res) => println!("success"),
///   Err(err) => eprintln!("error: {:?}", err),
/// }
/// # })
/// ```
#[derive(Debug)]
pub struct FileUploadCommand<'a, R> {
    pub filename: Cow<'a, str>,
    pub folder_id: u64,
    pub reader: R,
    pub no_partial: bool,
    pub part_size: usize,
}

impl<'a, R: Read + Send> FileUploadCommand<'a, R> {
    pub fn new<F: Into<Cow<'a, str>>>(filename: F, folder_id: u64, reader: R) -> Self {
        Self {
            filename: filename.into(),
            folder_id,
            reader,
            no_partial: false,
            part_size: DEFAULT_PART_SIZE,
        }
    }

    pub fn set_no_partial(&mut self, no_partial: bool) {
        self.no_partial = no_partial;
    }

    pub fn with_no_partial(mut self, no_partial: bool) -> Self {
        self.no_partial = no_partial;
        self
    }

    pub fn set_part_size(&mut self, part_size: usize) {
        self.part_size = part_size;
    }

    pub fn with_part_size(mut self, part_size: usize) -> Self {
        self.part_size = part_size;
        self
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::{FileUploadCommand, MultipartFileUploadCommand, MultipartFileUploadResponse};
    use crate::client::HttpClient;
    use crate::entry::File;
    use crate::error::Error;
    use crate::file::FileResponse;
    use crate::folder::FolderIdentifierParam;
    use crate::prelude::HttpCommand;
    use reqwest::multipart;
    use std::io::Read;

    #[async_trait::async_trait]
    impl HttpCommand for MultipartFileUploadCommand {
        type Output = Vec<File>;

        async fn execute(self, client: &HttpClient) -> Result<Self::Output, Error> {
            if self.entries.is_empty() {
                return Ok(Vec::new());
            }

            let mut form = multipart::Form::new();

            for (index, part) in self.entries.into_iter().enumerate() {
                let part_name = format!("f{index}");
                form = form.part(part_name, part);
            }

            client
                .post_request_multipart::<MultipartFileUploadResponse, _>(
                    "uploadfile",
                    FolderIdentifierParam::FolderId {
                        folderid: self.folder_id,
                    },
                    form,
                )
                .await
                .map(|res| res.metadata)
        }
    }

    #[derive(serde::Serialize)]
    struct CreateUploadParams {
        #[serde(
            rename = "nopartial",
            skip_serializing_if = "crate::client::is_false",
            serialize_with = "crate::client::serialize_bool"
        )]
        no_partial: bool,
    }

    #[derive(serde::Serialize)]
    struct UploadWriteParams {
        #[serde(rename = "uploadid")]
        upload_id: u64,
        #[serde(rename = "uploadoffset")]
        offset: usize,
    }

    #[derive(serde::Serialize)]
    struct UploadSaveParams<'a> {
        #[serde(rename = "uploadid")]
        upload_id: u64,
        name: &'a str,
        #[serde(rename = "folderid")]
        folder_id: u64,
    }

    #[async_trait::async_trait]
    impl<'a, R: Read + Send> HttpCommand for FileUploadCommand<'a, R> {
        type Output = File;

        async fn execute(self, client: &HttpClient) -> Result<File, Error> {
            let upload_id = client
                .get_request::<CreateUploadPayload, _>(
                    "upload_create",
                    CreateUploadParams {
                        no_partial: self.no_partial,
                    },
                )
                .await
                .map(|res| res.upload_id)?;

            let mut reader = ChunkReader::new(self.reader, self.part_size);

            while let (offset, Some(chunk)) = reader.next_chunk()? {
                client
                    .put_request_data::<(), _>(
                        "upload_write",
                        UploadWriteParams { upload_id, offset },
                        chunk,
                    )
                    .await?;
            }

            client
                .get_request::<FileResponse, _>(
                    "upload_save",
                    UploadSaveParams {
                        upload_id,
                        name: self.filename.as_ref(),
                        folder_id: self.folder_id,
                    },
                )
                .await
                .map(|item| item.metadata)
        }
    }

    #[derive(Debug, serde::Deserialize)]
    struct CreateUploadPayload {
        #[serde(rename = "uploadid")]
        upload_id: u64,
    }

    struct ChunkReader<R> {
        read: R,
        offset: usize,
        size: usize,
    }

    impl<R: Read> ChunkReader<R> {
        pub fn new(read: R, size: usize) -> Self {
            Self {
                read,
                offset: 0,
                size,
            }
        }

        pub fn next_chunk(&mut self) -> Result<(usize, Option<Vec<u8>>), Error> {
            let mut chunk = Vec::with_capacity(self.size);
            match self
                .read
                .by_ref()
                .take(chunk.capacity() as u64)
                .read_to_end(&mut chunk)
            {
                Ok(n) => {
                    let offset = self.offset;
                    self.offset += n;
                    if n != 0 {
                        Ok((offset, Some(chunk)))
                    } else {
                        Ok((offset, None))
                    }
                }
                Err(e) => Err(Error::Upload(e)),
            }
        }
    }
}

#[cfg(all(test, feature = "client-http"))]
mod http_tests {
    use super::{FileUploadCommand, MultipartFileUploadCommand};
    use crate::client::HttpClient;
    use crate::credentials::Credentials;
    use crate::prelude::HttpCommand;
    use crate::region::Region;
    use mockito::Matcher;

    #[tokio::test]
    async fn multipart_success() {
        crate::tests::init();
        let mut server = mockito::Server::new_async().await;
        let m_upload = server
            .mock("POST", "/uploadfile")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
            ]))
            .match_body(Matcher::Any)
            .match_header("accept", "*/*")
            .match_header("user-agent", crate::client::USER_AGENT)
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
        let creds = Credentials::access_token("access-token");
        let dc = Region::new(server.url());
        let api = HttpClient::new(creds, dc);
        //
        let file = tokio::fs::File::open("./readme.md").await.unwrap();
        let length = std::fs::metadata("./readme.md").unwrap().len();
        let cmd = MultipartFileUploadCommand::new(0).with_body_entry("big-file.bin", length, file);
        let result = cmd.execute(&api).await.unwrap();
        //
        assert_eq!(result.len(), 1);
        m_upload.assert();
    }

    #[tokio::test]
    async fn success() {
        crate::tests::init();
        let mut server = mockito::Server::new_async().await;
        let m_create = server
            .mock("GET", "/upload_create")
            .match_query(Matcher::UrlEncoded(
                "access_token".into(),
                "access-token".into(),
            ))
            .with_status(200)
            .with_body(r#"{ "result": 0, "uploadid": 42 }"#)
            .create();
        let m_write = server
            .mock("PUT", "/upload_write")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("uploadid".into(), "42".into()),
                Matcher::UrlEncoded("uploadoffset".into(), "0".into()),
            ]))
            .match_body(Matcher::Any)
            .with_status(200)
            .with_body(r#"{ "result": 0 }"#)
            .create();
        let m_save = server
            .mock("GET", "/upload_save")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("uploadid".into(), "42".into()),
                Matcher::UrlEncoded("name".into(), "testing.txt".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
            ]))
            .with_status(200)
            .with_body(
                r#"{
    "result": 0,
    "metadata": {
        "name": "testing.txt",
        "created": "Fri, 23 Jul 2021 19:39:14 +0000",
        "thumb": false,
        "modified": "Fri, 23 Jul 2021 19:39:14 +0000",
        "isfolder": false,
        "fileid": 5251776407,
        "hash": 10959076480325710862,
        "category": 0,
        "id": "f5251776407",
        "isshared": false,
        "ismine": true,
        "size": 10485760,
        "parentfolderid": 1073906698,
        "contenttype": "application\/octet-stream",
        "icon": "file"
    }
}"#,
            )
            .create();

        let creds = Credentials::access_token("access-token");
        let dc = Region::new(server.url());
        let api = HttpClient::new(creds, dc);
        //
        let cursor = std::io::Cursor::new("hello world!");
        let result = FileUploadCommand::new("testing.txt", 0, cursor)
            .with_no_partial(true)
            .execute(&api)
            .await
            .unwrap();
        //
        assert_eq!(result.base.name, "testing.txt");
        m_create.assert();
        m_write.assert();
        m_save.assert();
    }
}
