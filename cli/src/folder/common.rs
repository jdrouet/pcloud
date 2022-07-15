use async_recursion::async_recursion;
use pcloud::entry::{Entry, Folder};
use pcloud::error::Error;
use pcloud::http::HttpClient;
use pcloud::prelude::HttpCommand;
use sha2::{Digest, Sha256};
use std::path::Path;
use std::str::FromStr;

/// returns the content of a folder with retry mechanism
#[async_recursion]
pub(crate) async fn try_get_folder(
    pcloud: &HttpClient,
    folder_id: u64,
    retries: usize,
) -> Result<Folder, Error> {
    tracing::info!("loading folder, {} retries left", retries);
    match pcloud::folder::list::FolderListCommand::new(folder_id.into())
        .execute(pcloud)
        .await
    {
        Err(err) if retries > 0 => {
            tracing::warn!("unable to load folder: {:?}", err);
            try_get_folder(pcloud, folder_id, retries - 1).await
        }
        other => other,
    }
}

/// returns the content of a folder with retry mechanism
pub(crate) async fn try_get_folder_content(
    pcloud: &HttpClient,
    folder_id: u64,
    retries: usize,
) -> Result<Vec<Entry>, Error> {
    try_get_folder(pcloud, folder_id, retries)
        .await
        .map(|folder| folder.contents.unwrap_or_default())
}

#[async_recursion]
pub async fn try_get_file_checksum(
    pcloud: &HttpClient,
    file_id: u64,
    retries: usize,
) -> Result<String, Error> {
    tracing::info!("loading file checksum, {} retries left", retries);
    match pcloud::file::checksum::FileCheckSumCommand::new(file_id.into())
        .execute(pcloud)
        .await
    {
        Err(err) if retries > 0 => {
            tracing::warn!("unable to load file checksum: {:?}", err);
            try_get_file_checksum(pcloud, file_id, retries - 1).await
        }
        Err(err) => Err(err),
        Ok(res) => Ok(res.sha256),
    }
}

/// Computes the sha256 checksum of a local file
pub(crate) fn get_checksum(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path)
        .map_err(|err| format!("unable to open file {:?}: {:?}", path, err))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)
        .map_err(|err| format!("unable to compute hash for {:?}: {:?}", path, err))?;
    Ok(hex::encode(hasher.finalize()))
}

/// Method to compare existing files
#[derive(Clone, Copy)]
pub(crate) enum CompareMethod {
    /// Compute the checksum of the existing file and compares it with the remote file
    Checksum,
    /// Force upload event if the file already exists
    Force,
    /// Just checks the presence of the file, do not compare anything
    Presence,
}

impl FromStr for CompareMethod {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "checksum" => Ok(Self::Checksum),
            "force" => Ok(Self::Force),
            "presence" => Ok(Self::Presence),
            _ => Err(format!("invalid comparison method {:?}", value)),
        }
    }
}
