use pcloud::error::Error;
use pcloud::http::{HttpClient, HttpClientBuilder};
use pcloud::prelude::HttpCommand;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::collections::HashSet;
use std::fs::File;
use std::io::Write;
use std::path::{Path, PathBuf};
use temp_dir::TempDir;

pub(crate) fn init() {
    if let Err(err) = tracing_subscriber::fmt()
        .with_env_filter("pcloud_cli=trace")
        .try_init()
    {
        tracing::debug!("tracer error: {:?}", err);
    }
}

pub(crate) fn random_name() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

pub(crate) fn random_binary() -> Vec<u8> {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(256)
        .collect()
}

pub(crate) fn create_root() -> TempDir {
    TempDir::new().unwrap()
}

pub(crate) fn create_client() -> HttpClient {
    HttpClientBuilder::from_env().build().unwrap()
}

pub(crate) fn create_local_dir(parent: &Path, name: &str) -> PathBuf {
    let child = parent.join(name);
    std::fs::create_dir_all(&child).expect("couldn't create child folder");
    child
}

pub(crate) fn create_local_file(parent: &Path, name: &str) -> PathBuf {
    let child = parent.join(name);
    let mut file = File::create(&child).unwrap();
    writeln!(&mut file, "Hello World!").unwrap();
    child
}

fn flatten_remote(res: &mut HashSet<String>, path: &Path, folder: &pcloud::entry::Folder) {
    if let Some(ref children) = folder.contents {
        for child in children.iter() {
            match child {
                pcloud::entry::Entry::Folder(child) => {
                    let folder_path = path.join(child.base.name.as_str());
                    flatten_remote(res, &folder_path, child)
                }
                pcloud::entry::Entry::File(child) => {
                    if let Some(name) = path.join(child.base.name.as_str()).to_str() {
                        res.insert(name.to_string());
                    }
                }
            }
        }
    }
}

pub(crate) async fn scan_remote_dir(
    client: &HttpClient,
    folder_id: u64,
) -> Result<HashSet<String>, Error> {
    let folder = pcloud::folder::list::FolderListCommand::new(folder_id.into())
        .recursive(true)
        .execute(client)
        .await?;
    let root = PathBuf::from("/");
    let mut result = HashSet::new();
    flatten_remote(&mut result, &root, &folder);
    Ok(result)
}

pub(crate) async fn create_remote_dir(
    client: &HttpClient,
    parent_id: u64,
) -> Result<pcloud::entry::Folder, Error> {
    pcloud::folder::create::FolderCreateCommand::new(crate::tests::random_name(), parent_id)
        .execute(client)
        .await
}

pub(crate) async fn create_remote_file(
    client: &HttpClient,
    folder_id: u64,
) -> Result<pcloud::entry::File, Error> {
    let filename = format!("{}.bin", random_name());
    let binary = random_binary();
    pcloud::file::upload::FileUploadCommand::new(filename.as_str(), folder_id, binary.as_slice())
        .execute(client)
        .await
}

pub(crate) async fn delete_remote_dir(client: &HttpClient, folder_id: u64) -> Result<(), Error> {
    let _ = pcloud::folder::delete::FolderDeleteCommand::new(folder_id.into())
        .recursive(true)
        .execute(client)
        .await?;
    Ok(())
}
