//! Resources needed to upload a file

use std::io::Read;

/// Default size for splitting into chunks
pub const DEFAULT_PART_SIZE: usize = 10485760;

#[derive(Debug)]
#[cfg(feature = "client-http")]
pub struct MultipartFileUploadCommand {
    pub entries: Vec<(String, reqwest::Body)>,
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

    /// This method doesn't work because the file size is not known and therefore
    /// the content-length is not populated.
    // pub fn add_tokio_file_entry(self, filename: String, file: tokio::fs::File) -> Self {
    //     tracing::warn!(
    //         "handle tokio file does not work, the content-length header will not be set."
    //     );
    //     self.add_entry(filename, Body::from(file))
    // }

    pub fn add_sync_file_entry(
        self,
        filename: String,
        file: std::fs::File,
    ) -> Result<Self, std::io::Error> {
        self.add_sync_read_entry(filename, std::io::BufReader::new(file))
    }

    pub fn add_sync_read_entry<R: Read>(
        self,
        filename: String,
        mut reader: R,
    ) -> Result<Self, std::io::Error> {
        let mut buffer = Vec::new();
        reader.read_to_end(&mut buffer)?;
        Ok(self.add_entry(filename, reqwest::Body::from(buffer)))
    }

    pub fn add_entry(mut self, filename: String, body: reqwest::Body) -> Self {
        self.entries.push((filename, body));
        self
    }
}

/// Command to upload one or multiple files to a defined folder and name
///
/// Executing this command will return a `Vec` of [`File`](crate::entry::File) on success.
///
/// [More about it on the documentation](https://docs.pcloud.com/methods/file/uploadfile.html).
///
/// # Example using the [`HttpClient`](crate::http::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::http::HttpClientBuilder;
/// use pcloud::prelude::HttpCommand;
/// use pcloud::file::upload::MultipartFileUploadCommand;
/// use std::fs::File;
///
/// # tokio_test::block_on(async {
/// let file = File::open("Cargo.toml").unwrap();
/// let client = HttpClientBuilder::from_env().build().unwrap();
/// let cmd = MultipartFileUploadCommand::new(12)
///     .add_sync_file_entry("Cargo.toml".into(), file)
///     .unwrap();
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
/// # Example using the [`HttpClient`](crate::http::HttpClient)
///
/// To use this, the `client-http` feature should be enabled.
///
/// ```
/// use pcloud::http::HttpClientBuilder;
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
    pub filename: &'a str,
    pub folder_id: u64,
    pub reader: R,
    pub no_partial: bool,
    pub part_size: usize,
}

impl<'a, R: Read + Send> FileUploadCommand<'a, R> {
    pub fn new(filename: &'a str, folder_id: u64, reader: R) -> Self {
        Self {
            filename,
            folder_id,
            reader,
            no_partial: false,
            part_size: DEFAULT_PART_SIZE,
        }
    }

    pub fn no_partial(mut self, no_partial: bool) -> Self {
        self.no_partial = no_partial;
        self
    }

    pub fn part_size(mut self, part_size: usize) -> Self {
        self.part_size = part_size;
        self
    }
}

#[cfg(feature = "client-http")]
mod http {
    use super::{FileUploadCommand, MultipartFileUploadCommand, MultipartFileUploadResponse};
    use crate::entry::File;
    use crate::error::Error;
    use crate::file::FileResponse;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::request::Response;
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

            for (index, (filename, body)) in self.entries.into_iter().enumerate() {
                let part_name = format!("f{index}");
                let part = multipart::Part::stream(body).file_name(filename);
                form = form.part(part_name, part);
            }

            let params = vec![("folderid", self.folder_id.to_string())];
            let result: Response<MultipartFileUploadResponse> = client
                .post_request_multipart("uploadfile", &params, form)
                .await?;

            Ok(result.payload()?.metadata)
        }
    }

    #[async_trait::async_trait]
    impl<'a, R: Read + Send> HttpCommand for FileUploadCommand<'a, R> {
        type Output = File;

        async fn execute(self, client: &HttpClient) -> Result<File, Error> {
            let params = if self.no_partial {
                vec![("nopartial", 1.to_string())]
            } else {
                Vec::new()
            };
            let result: Response<CreateUploadPayload> =
                client.get_request("upload_create", &params).await?;
            let upload_id = result.payload().map(|item| item.upload_id)?;

            let mut reader = ChunkReader::new(self.reader, self.part_size);

            let upload_id_str = upload_id.to_string();

            while let (offset, Some(chunk)) = reader.next_chunk()? {
                let offset = offset.to_string();
                let params = vec![
                    ("uploadid", upload_id_str.to_string()),
                    ("uploadoffset", offset.to_string()),
                ];
                let response: Response<()> = client
                    .put_request_data("upload_write", &params, chunk)
                    .await?;
                response.payload()?;
            }

            let params = vec![
                ("uploadid", upload_id.to_string()),
                ("name", self.filename.to_string()),
                ("folderid", self.folder_id.to_string()),
            ];
            let result: Response<FileResponse> = client.get_request("upload_save", &params).await?;
            result.payload().map(|item| item.metadata)
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
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::region::Region;
    use mockito::{mock, Matcher};
    use std::fs::File;

    #[tokio::test]
    async fn multipart_success() {
        crate::tests::init();
        let m_upload = mock("POST", "/uploadfile")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("folderid".into(), "0".into()),
            ]))
            .match_body(Matcher::Any)
            .match_header("accept", "*/*")
            .match_header("user-agent", crate::http::USER_AGENT)
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
        let creds = Credentials::AccessToken("access-token".into());
        let dc = Region::mock();
        let api = HttpClient::new(creds, dc);
        //
        let file = File::open("./readme.md").unwrap();
        let cmd = MultipartFileUploadCommand::new(0)
            .add_sync_file_entry("big-file.bin".to_string(), file)
            .unwrap();
        let result = cmd.execute(&api).await.unwrap();
        //
        assert_eq!(result.len(), 1);
        m_upload.assert();
    }

    #[tokio::test]
    async fn success() {
        crate::tests::init();
        let m_create = mock("GET", "/upload_create")
            .match_query(Matcher::UrlEncoded(
                "access_token".into(),
                "access-token".into(),
            ))
            .with_status(200)
            .with_body(r#"{ "result": 0, "uploadid": 42 }"#)
            .create();
        let m_write = mock("PUT", "/upload_write")
            .match_query(Matcher::AllOf(vec![
                Matcher::UrlEncoded("access_token".into(), "access-token".into()),
                Matcher::UrlEncoded("uploadid".into(), "42".into()),
                Matcher::UrlEncoded("uploadoffset".into(), "0".into()),
            ]))
            .match_body(Matcher::Any)
            .with_status(200)
            .with_body(r#"{ "result": 0 }"#)
            .create();
        let m_save = mock("GET", "/upload_save")
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

        let creds = Credentials::AccessToken("access-token".into());
        let dc = Region::mock();
        let api = HttpClient::new(creds, dc);
        //
        let cursor = std::io::Cursor::new("hello world!");
        let cmd = FileUploadCommand::new("testing.txt", 0, cursor).no_partial(true);
        let result = cmd.execute(&api).await.unwrap();
        //
        assert_eq!(result.base.name, "testing.txt");
        m_create.assert();
        m_write.assert();
        m_save.assert();
    }
}
