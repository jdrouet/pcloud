use super::common::{get_checksum, try_get_file_checksum, try_get_folder, CompareMethod};
use async_recursion::async_recursion;
use clap::Parser;
use pcloud::entry::{Entry, Folder};
use pcloud::error::Error as PCloudError;
use pcloud::http::HttpClient;
use pcloud::prelude::HttpCommand;
use std::collections::HashMap;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};

fn read_local_folder(path: &Path) -> Vec<(String, PathBuf)> {
    tracing::debug!("{:?} reading local folder", path);
    match std::fs::read_dir(path) {
        Ok(list) => list
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter_map(|entry| {
                entry
                    .file_name()
                    .and_then(|fname| fname.to_str())
                    .map(String::from)
                    .map(|fname| (fname, entry))
            })
            .collect(),
        Err(err) => {
            tracing::warn!("{:?} unable to read directory: {:?}", path, err);
            Vec::new()
        }
    }
}

#[async_recursion]
async fn try_create_folder(
    client: &HttpClient,
    remote_path: &Path,
    parent_folder: u64,
    fname: &str,
    retries: usize,
) -> Result<Folder, Error> {
    tracing::info!("{:?} creating", remote_path);
    match pcloud::folder::create::FolderCreateCommand::new(fname.to_string(), parent_folder)
        .execute(client)
        .await
    {
        Err(err) if retries > 0 => {
            tracing::warn!("{:?} unable to create folder: {:?}", remote_path, err);
            try_create_folder(client, remote_path, parent_folder, fname, retries - 1).await
        }
        Err(err) => Err(err.into()),
        Ok(res) => Ok(res),
    }
}

#[async_recursion]
async fn try_upload_file(
    pcloud: &HttpClient,
    remote_path: &Path,
    local_path: &Path,
    folder_id: u64,
    fname: &str,
    retries: usize,
) -> Result<(), Error> {
    tracing::info!("{:?} uploading", remote_path);
    let reader = std::fs::File::open(local_path).unwrap();
    match pcloud::file::upload::FileUploadCommand::new(fname, folder_id, reader)
        .no_partial(false)
        .execute(pcloud)
        .await
    {
        Err(err) if retries > 0 => {
            tracing::warn!("{:?} unable to upload file: {:?}", remote_path, err);
            try_upload_file(
                pcloud,
                remote_path,
                local_path,
                folder_id,
                fname,
                retries - 1,
            )
            .await
        }
        Err(err) => Err(err.into()),
        Ok(_) => Ok(()),
    }
}

struct FileUploader {
    remote_path: PathBuf,
    remote_existing_id: Option<u64>,
    remote_folder_id: u64,
    filename: String,
    local_path: PathBuf,
}

impl FileUploader {
    async fn execute(
        self,
        client: &HttpClient,
        compare_method: CompareMethod,
        retries: usize,
    ) -> Result<(), Error> {
        if compare_method
            .should_upload_file(
                client,
                &self.remote_path,
                &self.remote_existing_id,
                &self.local_path,
                retries,
            )
            .await?
        {
            try_upload_file(
                client,
                &self.remote_path,
                &self.local_path,
                self.remote_folder_id,
                self.filename.as_str(),
                retries,
            )
            .await?;
        }
        Ok(())
    }
}

enum RemoteFolder {
    Existing(u64),
    Missing(u64, String),
}

impl RemoteFolder {
    async fn get(
        &self,
        client: &HttpClient,
        remote_path: &Path,
        retries: usize,
    ) -> Result<Folder, Error> {
        let folder_id = match self {
            Self::Existing(folder_id) => *folder_id,
            Self::Missing(parent_folder_id, filename) => {
                let created_folder =
                    try_create_folder(client, remote_path, *parent_folder_id, filename, retries)
                        .await?;
                created_folder.folder_id
            }
        };
        Ok(try_get_folder(client, remote_path, folder_id, retries).await?)
    }
}

struct FolderVisitor {
    remote_path: PathBuf,
    remote_folder: RemoteFolder,
    local_path: PathBuf,
}

impl FolderVisitor {
    #[tracing::instrument(name = "folder_visitor", skip_all)]
    async fn execute(
        self,
        client: &HttpClient,
        excludes: &[glob::Pattern],
        retries: usize,
        queue: async_channel::Sender<FileUploader>,
    ) -> Result<Vec<FolderVisitor>, Error> {
        let folder = self
            .remote_folder
            .get(client, &self.remote_path, retries)
            .await?;
        let remote_content: HashMap<&str, &Entry> = folder
            .contents
            .as_ref()
            .map(|items| {
                items
                    .iter()
                    .map(|entry| (entry.base().name.as_str(), entry))
                    .collect()
            })
            .unwrap_or_default();
        let mut result = Vec::new();
        for (fname, local_path) in read_local_folder(&self.local_path) {
            let new_remote_path = self.remote_path.join(&fname);
            if excludes.iter().any(|p| p.matches_path(&new_remote_path)) {
                tracing::info!(
                    "{:?} is matching an exclusion pattern, ignoring",
                    new_remote_path
                );
                continue;
            }
            if local_path.is_dir() {
                let remote_folder = remote_content
                    .get(fname.as_str())
                    .and_then(|entry| entry.as_folder())
                    .map(|folder| RemoteFolder::Existing(folder.folder_id))
                    .unwrap_or_else(|| RemoteFolder::Missing(folder.folder_id, fname));
                result.push(FolderVisitor {
                    remote_path: new_remote_path,
                    remote_folder,
                    local_path,
                });
            } else if local_path.is_file() {
                if let Err(err) = queue
                    .send(FileUploader {
                        remote_path: new_remote_path.clone(),
                        remote_existing_id: remote_content
                            .get(fname.as_str())
                            .and_then(|entry| entry.as_file())
                            .map(|file| file.file_id),
                        remote_folder_id: folder.folder_id,
                        filename: fname,
                        local_path,
                    })
                    .await
                {
                    tracing::warn!(
                        "unable to add {:?} to the uploading queue: {err}",
                        new_remote_path
                    );
                }
            }
        }
        Ok(result)
    }
}

#[derive(Debug)]
enum Error {
    PCloud(PCloudError),
    Io(IoError),
}

impl From<PCloudError> for Error {
    fn from(err: PCloudError) -> Self {
        Self::PCloud(err)
    }
}

impl From<IoError> for Error {
    fn from(err: IoError) -> Self {
        Self::Io(err)
    }
}

async fn should_upload_file_with_checksum(
    pcloud: &HttpClient,
    remote_path: &Path,
    remote_id: &Option<u64>,
    local_path: &Path,
    retries: usize,
) -> Result<bool, Error> {
    if let Some(file_id) = remote_id {
        tracing::info!("{:?} already exists remotely", remote_path);
        match get_checksum(local_path) {
            Ok(checksum) => {
                let remote_checksum =
                    try_get_file_checksum(pcloud, remote_path, *file_id, retries).await?;
                if remote_checksum != checksum {
                    tracing::debug!("{:?} checksum mismatch, uploading again", remote_path);
                }
                Ok(remote_checksum != checksum)
            }
            Err(error) => {
                tracing::warn!("skipping upload, {}", error);
                Ok(false)
            }
        }
    } else {
        tracing::info!("{:?} missing remotely, uploading", remote_path);
        Ok(true)
    }
}

impl CompareMethod {
    async fn should_upload_file(
        &self,
        pcloud: &HttpClient,
        remote_path: &Path,
        remote_id: &Option<u64>,
        local_path: &Path,
        retries: usize,
    ) -> Result<bool, Error> {
        match self {
            Self::Checksum => {
                should_upload_file_with_checksum(
                    pcloud,
                    remote_path,
                    remote_id,
                    local_path,
                    retries,
                )
                .await
            }
            Self::Force => Ok(true),
            Self::Presence => Ok(remote_id.is_some()),
        }
    }
}

#[derive(Parser)]
pub struct Command {
    /// The used stategy to check if a file should be uploaded
    #[clap(long, default_value = "checksum")]
    compare_method: CompareMethod,
    /// Files to exclude from uploading
    #[clap(long)]
    exclude: Vec<glob::Pattern>,
    /// Number of allowed failure.
    #[clap(long, default_value_t = 5)]
    retries: usize,
    /// Number of uploads in parallel
    #[clap(long, default_value_t = 5)]
    uploader_count: usize,
    /// Capacity of the download queue
    #[clap(long, default_value_t = 1024)]
    upload_queue_capacity: usize,
    /// Local folder to synchronize.
    #[clap()]
    path: PathBuf,
}

impl Command {
    pub async fn execute(&self, client: HttpClient, folder_id: u64) {
        let (tx, rx) = async_channel::bounded::<FileUploader>(self.upload_queue_capacity);

        let mut uploaders = Vec::with_capacity(self.uploader_count);
        for _ in 0..self.uploader_count {
            let uploader_rx = rx.clone();
            let uploader_client = client.clone();
            let uploader_retries = self.retries;
            let uploader_compare_method = self.compare_method;

            uploaders.push(tokio::spawn(async move {
                while let Ok(next) = uploader_rx.recv().await {
                    if let Err(err) = next
                        .execute(&uploader_client, uploader_compare_method, uploader_retries)
                        .await
                    {
                        tracing::error!("unable to upload file: {:?}", err);
                    }
                    tracing::info!("uploading queue contains now {} items", uploader_rx.len());
                }
                tracing::debug!("uploading queue is closed and empty, closing thread");
            }));
        }

        let mut visitor_queue = vec![FolderVisitor {
            remote_path: PathBuf::from("/"),
            remote_folder: RemoteFolder::Existing(folder_id),
            local_path: self.path.clone(),
        }];
        while let Some(next) = visitor_queue.pop() {
            match next
                .execute(&client, &self.exclude, self.retries, tx.clone())
                .await
            {
                Ok(found) => visitor_queue.extend(found),
                Err(err) => tracing::error!("unable to visit folder: {:?}", err),
            };
        }
        tx.close();
        tracing::info!("visitor is done, waiting for the uploader to finish");
        for uploader_handler in uploaders {
            if let Err(err) = uploader_handler.await {
                tracing::error!("something wrong happened with the uploaders: {:?}", err);
            }
        }
    }
}

#[cfg(all(test, feature = "protected"))]
mod tests {
    use super::Command;
    use crate::folder::common::CompareMethod;
    use crate::tests::*;
    use std::path::{Path, PathBuf};

    fn build_cmd(root: &Path, exclude: Vec<&'static str>) -> Command {
        Command {
            compare_method: CompareMethod::Checksum,
            exclude: exclude
                .iter()
                .map(|value| glob::Pattern::new(value).unwrap())
                .collect(),
            retries: 5,
            path: PathBuf::from(root),
            uploader_count: 2,
            upload_queue_capacity: 64,
        }
    }

    #[tokio::test]
    async fn simple() {
        // prepare basic folder
        let root = create_root();
        let _root_file = create_local_file(root.path(), "foo.txt");
        let first = create_local_dir(root.path(), "first");
        let _first_file = create_local_file(&first, "foo.txt");
        let second = create_local_dir(&first, "second");
        let _second_file = create_local_file(&second, "foo.txt");
        //
        let client = create_client();
        let remote_root = create_remote_dir(&client, 0).await.unwrap();
        //
        build_cmd(root.path(), Vec::new())
            .execute(client.clone(), remote_root.folder_id)
            .await;
        //
        let remote_content = scan_remote_dir(&client, remote_root.folder_id)
            .await
            .unwrap();
        assert_eq!(remote_content.len(), 3);
        assert!(remote_content.contains("/foo.txt"));
        assert!(remote_content.contains("/first/foo.txt"));
        assert!(remote_content.contains("/first/second/foo.txt"));
        // add more files locally
        let _third_file = create_local_file(&second, "bar.txt");
        //
        build_cmd(root.path(), Vec::new())
            .execute(client.clone(), remote_root.folder_id)
            .await;
        //
        let remote_content = scan_remote_dir(&client, remote_root.folder_id)
            .await
            .unwrap();
        assert_eq!(remote_content.len(), 4);
        assert!(remote_content.contains("/foo.txt"));
        assert!(remote_content.contains("/first/foo.txt"));
        assert!(remote_content.contains("/first/second/foo.txt"));
        assert!(remote_content.contains("/first/second/bar.txt"));
        //
        delete_remote_dir(&client, remote_root.folder_id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn exclude_bin_files() {
        // prepare basic folder
        let root = create_root();
        let _root_file = create_local_file(root.path(), "foo.txt");
        let first = create_local_dir(root.path(), "first");
        let _first_file = create_local_file(&first, "foo.bin");
        let second = create_local_dir(&first, "second");
        let _second_file = create_local_file(&second, "foo.txt");
        //
        let client = create_client();
        let remote_root = create_remote_dir(&client, 0).await.unwrap();
        //
        build_cmd(root.path(), vec!["*.bin"])
            .execute(client.clone(), remote_root.folder_id)
            .await;
        //
        let remote_content = scan_remote_dir(&client, remote_root.folder_id)
            .await
            .unwrap();
        assert_eq!(remote_content.len(), 2);
        assert!(remote_content.contains("/foo.txt"));
        assert!(!remote_content.contains("/first/foo.bin"));
        assert!(remote_content.contains("/first/second/foo.txt"));
        // cleanup
        delete_remote_dir(&client, remote_root.folder_id)
            .await
            .unwrap();
    }
}
