pub mod copy;
pub mod download;
pub mod get_info;
pub mod get_link;
pub mod rename;
pub mod upload;

use crate::entry::RemoteEntry;

#[derive(Debug, serde::Deserialize)]
pub struct FileResponse {
    metadata: RemoteEntry,
}
