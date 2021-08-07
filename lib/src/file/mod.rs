pub mod copy;
pub mod delete;
pub mod download;
pub mod get_info;
pub mod get_link;
pub mod rename;
pub mod upload;

use crate::entry::File;

#[derive(Debug, serde::Deserialize)]
pub struct FileResponse {
    metadata: File,
}
