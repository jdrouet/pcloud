mod copy;
mod download;
mod get_info;
mod get_link;
mod rename;
mod upload;

use crate::common::RemoteFile;

#[derive(Debug, serde::Deserialize)]
pub struct FileResponse {
    metadata: RemoteFile,
}
