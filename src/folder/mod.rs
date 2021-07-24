pub mod create;
pub mod delete;
pub mod list;
pub mod rename;

use crate::entry::RemoteEntry;

pub const ROOT: usize = 0;

#[derive(Debug, serde::Deserialize)]
pub(crate) struct FolderResponse {
    metadata: RemoteEntry,
}
