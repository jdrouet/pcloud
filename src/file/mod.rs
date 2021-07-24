pub mod copy;
pub mod download;
pub mod get_info;
pub mod get_link;
pub mod rename;
pub mod upload;

use crate::common::RemoteFile;

#[derive(Debug, serde::Deserialize)]
pub struct FileResponse {
    metadata: RemoteFile,
}
