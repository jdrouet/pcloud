use super::FileResponse;
use crate::entry::File;
use crate::error::Error;
use crate::http::HttpClient;
use crate::request::Response;
use std::io::Read;

#[derive(Debug, serde::Deserialize)]
struct CreateUploadPayload {
    #[serde(rename = "uploadid")]
    upload_id: usize,
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

impl HttpClient {
    async fn create_upload_file(&self) -> Result<usize, Error> {
        let result: Response<CreateUploadPayload> =
            self.get_request("upload_create", &Vec::new()).await?;
        result.payload().map(|item| item.upload_id)
    }

    async fn write_chunk_file(
        &self,
        params: &[(&str, String)],
        chunk: Vec<u8>,
    ) -> Result<(), Error> {
        let response: Response<()> = self.put_request_data("upload_write", params, chunk).await?;
        response.payload()?;
        Ok(())
    }

    async fn save_file(
        &self,
        upload_id: usize,
        filename: &str,
        folder_id: usize,
    ) -> Result<File, Error> {
        let params = vec![
            ("uploadid", upload_id.to_string()),
            ("name", filename.to_string()),
            ("folderid", folder_id.to_string()),
        ];
        let result: Response<FileResponse> = self.get_request("upload_save", &params).await?;
        result.payload().map(|item| item.metadata)
    }

    /// Upload a file in a folder
    ///
    /// # Arguments
    ///
    /// * `input` - File to read from.
    /// * `filename` - Name of the file to create.
    /// * `folder_id` - ID of the folder where to upload the file.
    ///
    #[tracing::instrument(skip(self, input))]
    pub async fn upload_file<R: Read>(
        &self,
        input: R,
        filename: &str,
        folder_id: usize,
    ) -> Result<File, Error> {
        let upload_id = self.create_upload_file().await?;
        let mut reader = ChunkReader::new(input, self.upload_part_size);

        let upload_id_str = upload_id.to_string();

        while let (offset, Some(chunk)) = reader.next_chunk()? {
            let offset = offset.to_string();
            let params = vec![
                ("uploadid", upload_id_str.to_string()),
                ("uploadoffset", offset.to_string()),
            ];
            self.write_chunk_file(&params, chunk).await?;
        }

        self.save_file(upload_id, filename, folder_id).await
    }
}

#[cfg(test)]
mod tests {
    use crate::credentials::Credentials;
    use crate::http::HttpClient;
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
        let result = api.upload_file(cursor, "testing.txt", 0).await.unwrap();
        assert_eq!(result.base.name, "testing.txt");
        m_create.assert();
        m_write.assert();
        m_save.assert();
    }
}
