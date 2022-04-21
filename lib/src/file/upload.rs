use super::FileResponse;
use crate::entry::File;
use crate::error::Error;
use crate::http::HttpClient;
use crate::prelude::HttpCommand;
use crate::request::Response;
use std::io::Read;

pub const DEFAULT_PART_SIZE: usize = 10485760;

#[derive(Debug)]
pub struct FileUploadCommand<'a, R> {
    filename: &'a str,
    folder_id: u64,
    reader: R,
    no_partial: bool,
    part_size: usize,
}

impl<'a, R: Read> FileUploadCommand<'a, R> {
    pub fn new(filename: &'a str, folder_id: u64, reader: R) -> Self {
        Self {
            filename,
            folder_id,
            reader,
            no_partial: false,
            part_size: DEFAULT_PART_SIZE,
        }
    }

    pub fn set_no_partial(&mut self, no_partial: bool) {
        self.no_partial = no_partial;
    }

    pub fn set_part_size(&mut self, part_size: usize) {
        self.part_size = part_size;
    }

    async fn create_upload_file(&self, client: &HttpClient) -> Result<u64, Error> {
        let params = if self.no_partial {
            vec![("nopartial", 1.to_string())]
        } else {
            Vec::new()
        };
        let result: Response<CreateUploadPayload> =
            client.get_request("upload_create", &params).await?;
        result.payload().map(|item| item.upload_id)
    }
}

#[async_trait::async_trait(?Send)]
impl<'a, R: Read> HttpCommand for FileUploadCommand<'a, R> {
    type Output = File;

    async fn execute(self, client: &HttpClient) -> Result<File, Error> {
        let upload_id = self.create_upload_file(client).await?;
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

#[cfg(test)]
mod tests {
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
        let mut cmd = FileUploadCommand::new("testing.txt", 0, cursor);
        cmd.set_no_partial(true);
        let result = cmd.execute(&api).await.unwrap();
        //
        assert_eq!(result.base.name, "testing.txt");
        m_create.assert();
        m_write.assert();
        m_save.assert();
    }
}
