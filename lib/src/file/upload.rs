//! Resources needed to upload a file

use std::io::Read;

/// Default size for splitting into chunks
pub const DEFAULT_PART_SIZE: usize = 10485760;

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
    use super::FileUploadCommand;
    use crate::entry::File;
    use crate::error::Error;
    use crate::file::FileResponse;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::request::Response;
    use std::io::Read;

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
    use super::FileUploadCommand;
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
    use crate::prelude::HttpCommand;
    use crate::region::Region;
    use mockito::{mock, Matcher};

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
