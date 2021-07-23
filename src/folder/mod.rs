mod create;
mod delete;
mod list;
mod rename;

use crate::common::RemoteFile;

pub const ROOT: usize = 0;

#[derive(Debug, serde::Deserialize)]
pub struct FolderResponse {
    metadata: RemoteFile,
}
