use crate::common::RemoteFile;
use crate::request::{Error, Response};
use crate::PCloudApi;
use std::io::Read;

#[derive(Debug, serde::Deserialize)]
struct CreateUploadFilePayload {
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

#[derive(Debug, serde::Deserialize)]
struct ResponseUploadFile {
    metadata: RemoteFile,
}

impl PCloudApi {
    async fn create_upload_file(&self) -> Result<CreateUploadFilePayload, Error> {
        let result: Response<CreateUploadFilePayload> =
            self.get_request("upload_create", &Vec::new()).await?;
        result.payload()
    }

    async fn write_chunk_file(&self, params: &[(&str, &str)], chunk: Vec<u8>) -> Result<(), Error> {
        let response: Response<()> = self.put_request_data("upload_write", params, chunk).await?;
        response.payload()?;
        Ok(())
    }

    async fn save_file(
        &self,
        upload_id: usize,
        filename: &str,
        folder_id: usize,
    ) -> Result<RemoteFile, Error> {
        let folder_id = folder_id.to_string();
        let upload_id = upload_id.to_string();
        let params = vec![
            ("uploadid", upload_id.as_str()),
            ("name", filename),
            ("folderid", folder_id.as_str()),
        ];
        let result: Response<ResponseUploadFile> = self.get_request("upload_save", &params).await?;
        result.payload().map(|item| item.metadata)
    }

    pub async fn upload_file<R: Read>(
        &self,
        input: R,
        filename: &str,
        folder_id: usize,
    ) -> Result<RemoteFile, Error> {
        let upload = self.create_upload_file().await?;
        let mut reader = ChunkReader::new(input, self.upload_part_size);

        let upload_id = upload.upload_id.to_string();

        while let (offset, Some(chunk)) = reader.next_chunk()? {
            let offset = offset.to_string();
            let params = vec![
                ("uploadid", upload_id.as_str()),
                ("uploadoffset", offset.as_str()),
            ];
            self.write_chunk_file(&params, chunk).await?;
        }

        self.save_file(upload.upload_id, filename, folder_id).await
    }
}
