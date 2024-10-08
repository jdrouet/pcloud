use pcloud::entry::Folder;
use pcloud::prelude::HttpCommand;
use pcloud::{client::HttpClient, entry::File};

pub(crate) async fn maybe_get_file(
    client: &HttpClient,
    path: &str,
) -> Result<Option<(File, String)>, pcloud::error::Error> {
    let file_id = pcloud::file::FileIdentifier::path(path);
    let result = pcloud::file::checksum::FileCheckSumCommand::new(file_id)
        .execute(client)
        .await;
    match result {
        Ok(inner) => Ok(Some((inner.metadata, inner.sha1))),
        Err(pcloud::error::Error::Protocol(2009, _)) => Ok(None),
        Err(inner) => Err(inner),
    }
}

pub(crate) async fn get_folder(
    client: &HttpClient,
    path: &str,
) -> Result<Folder, pcloud::error::Error> {
    pcloud::folder::list::FolderListCommand::new(path.into())
        .execute(client)
        .await
}

pub(crate) async fn maybe_get_folder(
    client: &HttpClient,
    path: &str,
) -> Result<Option<Folder>, pcloud::error::Error> {
    match get_folder(client, path).await {
        Ok(inner) => Ok(Some(inner)),
        Err(pcloud::error::Error::Protocol(2005, _)) => Ok(None),
        Err(inner) => Err(inner),
    }
}
