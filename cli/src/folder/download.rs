use super::common::{get_checksum, CompareMethod};
use async_recursion::async_recursion;
use clap::Parser;
use pcloud::entry::{Entry, File};
use pcloud::error::Error as PCloudError;
use pcloud::file::checksum::FileCheckSumCommand;
use pcloud::file::download::FileDownloadCommand;
use pcloud::folder::list::FolderListCommand;
use pcloud::http::HttpClient;
use pcloud::prelude::HttpCommand;
use std::fs;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};
use std::sync::{atomic::AtomicBool, Arc};
use tokio::sync::mpsc;

#[async_recursion]
async fn try_get_folder_content(
    pcloud: &HttpClient,
    remote_path: &Path,
    folder_id: u64,
    retries: usize,
) -> Result<Vec<Entry>, Error> {
    tracing::info!(
        "{:?} loading folder content, {} retries left",
        remote_path,
        retries
    );
    match FolderListCommand::new(folder_id.into())
        .execute(pcloud)
        .await
    {
        Err(err) if retries > 0 => {
            tracing::warn!("{:?} unable to load folder content: {:?}", remote_path, err);
            try_get_folder_content(pcloud, remote_path, folder_id, retries - 1).await
        }
        Err(err) => Err(err.into()),
        Ok(res) => Ok(res.contents.unwrap_or_default()),
    }
}

#[async_recursion]
async fn try_get_file_checksum(
    pcloud: &HttpClient,
    remote_path: &Path,
    file_id: u64,
    retries: usize,
) -> Result<String, Error> {
    tracing::info!(
        "{:?} loading file checksum, {} retries left",
        remote_path,
        retries
    );
    match FileCheckSumCommand::new(file_id.into())
        .execute(pcloud)
        .await
    {
        Err(err) if retries > 0 => {
            tracing::warn!("{:?} unable to load file checksum: {:?}", remote_path, err);
            try_get_file_checksum(pcloud, remote_path, file_id, retries - 1).await
        }
        Err(err) => Err(err.into()),
        Ok(res) => Ok(res.sha256),
    }
}

#[async_recursion]
async fn try_download_file(
    pcloud: &HttpClient,
    remote_path: &Path,
    file_id: u64,
    local_path: &Path,
    retries: usize,
) -> Result<(), Error> {
    tracing::info!("{:?} downloading to {:?}", remote_path, local_path);
    let file = fs::OpenOptions::new()
        .create(true)
        .write(true)
        .truncate(true)
        .open(&local_path)?;
    match FileDownloadCommand::new(file_id.into(), file)
        .execute(pcloud)
        .await
    {
        Err(err) if retries > 0 => {
            tracing::warn!("{:?} unable to download file: {:?}", remote_path, err);
            try_download_file(pcloud, remote_path, file_id, local_path, retries - 1).await
        }
        Err(err) => Err(err.into()),
        Ok(_) => Ok(()),
    }
}

async fn should_download_file_with_checksum(
    pcloud: &HttpClient,
    remote_path: &Path,
    remote_file: &File,
    local_path: &Path,
    retries: usize,
) -> Result<bool, Error> {
    if local_path.exists() {
        match get_checksum(local_path) {
            Ok(checksum) => {
                let remote_checksum =
                    try_get_file_checksum(pcloud, remote_path, remote_file.file_id, retries)
                        .await?;
                if remote_checksum != checksum {
                    tracing::debug!("{:?} checksum mismatch, downloading again", local_path);
                }
                Ok(remote_checksum != checksum)
            }
            Err(error) => {
                tracing::warn!("unable to compute checksum, forcing download: {}", error);
                Ok(true)
            }
        }
    } else {
        Ok(true)
    }
}

impl CompareMethod {
    async fn should_download_file(
        &self,
        pcloud: &HttpClient,
        remote_path: &Path,
        remote_file: &File,
        local_path: &Path,
        retries: usize,
    ) -> Result<bool, Error> {
        match self {
            Self::Checksum => {
                should_download_file_with_checksum(
                    pcloud,
                    remote_path,
                    remote_file,
                    local_path,
                    retries,
                )
                .await
            }
            Self::Force => Ok(true),
            Self::Presence => Ok(!local_path.exists()),
        }
    }
}
struct FileDownloader {
    remote_path: PathBuf,
    remote_file: File,
    local_path: PathBuf,
}

impl FileDownloader {
    #[tracing::instrument(name = "file_downloader", skip_all)]
    async fn execute(
        self,
        client: &HttpClient,
        compare_method: &CompareMethod,
        retries: usize,
    ) -> Result<(), Error> {
        if compare_method
            .should_download_file(
                client,
                &self.remote_path,
                &self.remote_file,
                &self.local_path,
                retries,
            )
            .await?
        {
            try_download_file(
                client,
                &self.remote_path,
                self.remote_file.file_id,
                &self.local_path,
                retries,
            )
            .await?;
        }
        Ok(())
    }
}

struct FolderVisitor {
    remote_path: PathBuf,
    remote_folder_id: u64,
    local_path: PathBuf,
}

impl FolderVisitor {
    #[tracing::instrument(name = "folder_visitor", skip_all)]
    async fn execute(
        self,
        client: &HttpClient,
        excludes: &[glob::Pattern],
        retries: usize,
        queue: mpsc::Sender<FileDownloader>,
    ) -> Result<Vec<FolderVisitor>, Error> {
        let mut results = Vec::new();
        for entry in
            try_get_folder_content(client, &self.remote_path, self.remote_folder_id, retries)
                .await?
        {
            let new_remote_path = self.remote_path.join(entry.base().name.as_str());
            if excludes.iter().any(|p| p.matches_path(&new_remote_path)) {
                tracing::info!(
                    "{:?} is matching an exclusion pattern, ignoring",
                    new_remote_path
                );
                continue;
            }
            match entry {
                Entry::File(file) => {
                    if let Err(err) = queue
                        .send(FileDownloader {
                            remote_path: new_remote_path,
                            local_path: self.local_path.join(file.base.name.as_str()),
                            remote_file: file,
                        })
                        .await
                    {
                        tracing::error!("unable to append file to download queue: {err}");
                    }
                }
                Entry::Folder(folder) => {
                    let new_remote_path = self.remote_path.join(folder.base.name.as_str());
                    let new_local_path = self.local_path.join(folder.base.name.as_str());
                    if let Err(err) = fs::create_dir_all(&new_local_path) {
                        tracing::warn!("unable to create folder {:?}: {:?}", new_local_path, err);
                    } else {
                        results.push(FolderVisitor {
                            remote_path: new_remote_path,
                            remote_folder_id: folder.folder_id,
                            local_path: new_local_path,
                        });
                    }
                }
            }
        }
        Ok(results)
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

#[derive(Parser)]
pub struct Command {
    /// The used stategy to check if a file should be downloaded
    #[clap(long, default_value = "checksum")]
    compare_method: CompareMethod,
    /// Files to exclude from downloading
    #[clap(long)]
    exclude: Vec<glob::Pattern>,
    /// Number of allowed failure.
    #[clap(long, default_value_t = 5)]
    retries: usize,
    /// Local folder to synchronize.
    #[clap()]
    path: PathBuf,
}

impl Command {
    pub async fn execute(&self, pcloud: HttpClient, folder_id: u64) {
        let (tx, mut rx) = mpsc::channel::<FileDownloader>(100);
        // value to set to false when the visitor is done
        let visiting = Arc::new(AtomicBool::new(true));

        let downloader_client = pcloud.clone();
        let downloader_retries = self.retries;
        let downloader_compare_method = self.compare_method;
        let downloader_visiting = Arc::clone(&visiting);
        let downloader_handler = tokio::spawn(async move {
            loop {
                if let Some(next) = rx.recv().await {
                    if let Err(err) = next
                        .execute(
                            &downloader_client,
                            &downloader_compare_method,
                            downloader_retries,
                        )
                        .await
                    {
                        tracing::error!("unable to download file: {:?}", err);
                    }
                } else if !downloader_visiting.load(std::sync::atomic::Ordering::Relaxed) {
                    tracing::info!("nothing in the download queue, visitor is finished, closing");
                    break;
                } else {
                    tracing::info!("nothing in the download queue, waiting 1s");
                    tokio::time::sleep(std::time::Duration::from_secs(1)).await;
                }
            }
        });
        let mut visitor_queue = Vec::new();
        visitor_queue.push(FolderVisitor {
            remote_path: PathBuf::from("/"),
            remote_folder_id: folder_id,
            local_path: self.path.clone(),
        });
        while let Some(next) = visitor_queue.pop() {
            match next
                .execute(&pcloud, &self.exclude, self.retries, tx.clone())
                .await
            {
                Ok(found) => visitor_queue.extend(found),
                Err(err) => tracing::error!("unable to visit folder: {:?}", err),
            };
        }
        visiting.store(false, std::sync::atomic::Ordering::Relaxed);
        tracing::info!("visitor is done, waiting for the downloader to finish");
        if let Err(err) = downloader_handler.await {
            tracing::error!("something wrong happened with the downloaders: {:?}", err);
        }
    }
}

#[cfg(all(test, feature = "protected"))]
mod tests {
    use super::Command;
    use crate::folder::common::CompareMethod;
    use crate::tests::*;
    use std::path::{Path, PathBuf};

    fn build_cmd(root: &Path, remove_after_download: bool, exclude: Vec<String>) -> Command {
        Command {
            remove_after_download,
            exclude: exclude
                .iter()
                .map(|item| glob::Pattern::new(item).unwrap())
                .collect(),
            compare_method: CompareMethod::Checksum,
            retries: 5,
            path: PathBuf::from(root),
        }
    }

    #[tokio::test]
    async fn simple() {
        // prepare basic folder
        let client = create_client();
        let remote_root = create_remote_dir(&client, 0).await.unwrap();
        let remote_file_first = create_remote_file(&client, remote_root.folder_id)
            .await
            .unwrap();
        let remote_folder_first = create_remote_dir(&client, remote_root.folder_id)
            .await
            .unwrap();
        let remote_file_second = create_remote_file(&client, remote_folder_first.folder_id)
            .await
            .unwrap();
        //
        let root = create_root();
        build_cmd(root.path(), false, vec![])
            .execute(client.clone(), remote_root.folder_id)
            .await;
        assert!(root
            .path()
            .join(remote_file_first.base.name.as_str())
            .exists());
        assert!(root
            .path()
            .join(remote_folder_first.base.name.as_str())
            .exists());
        assert!(root
            .path()
            .join(remote_folder_first.base.name.as_str())
            .join(remote_file_second.base.name.as_str())
            .exists());
        //
        let remote_file_third = create_remote_file(&client, remote_folder_first.folder_id)
            .await
            .unwrap();
        build_cmd(root.path(), false, vec![])
            .execute(client.clone(), remote_root.folder_id)
            .await;
        assert!(root
            .path()
            .join(remote_folder_first.base.name.as_str())
            .join(remote_file_third.base.name.as_str())
            .exists());
        //
        delete_remote_dir(&client, remote_root.folder_id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn with_exclude() {
        // prepare basic folder
        let client = create_client();
        let remote_root = create_remote_dir(&client, 0).await.unwrap();
        let remote_file_first = create_remote_file(&client, remote_root.folder_id)
            .await
            .unwrap();
        let remote_folder_first = create_remote_dir(&client, remote_root.folder_id)
            .await
            .unwrap();
        let remote_file_second = create_remote_file(&client, remote_folder_first.folder_id)
            .await
            .unwrap();
        let remote_file_third = create_remote_file(&client, remote_folder_first.folder_id)
            .await
            .unwrap();
        //
        let root = create_root();
        build_cmd(
            root.path(),
            false,
            vec![format!(
                "/{}/{}",
                remote_folder_first.base.name, remote_file_third.base.name
            )],
        )
        .execute(client.clone(), remote_root.folder_id)
        .await;
        assert!(root
            .path()
            .join(remote_file_first.base.name.as_str())
            .exists());
        assert!(root
            .path()
            .join(remote_folder_first.base.name.as_str())
            .exists());
        assert!(root
            .path()
            .join(remote_folder_first.base.name.as_str())
            .join(remote_file_second.base.name.as_str())
            .exists());
        assert!(!root
            .path()
            .join(remote_folder_first.base.name.as_str())
            .join(remote_file_third.base.name.as_str())
            .exists());
        //
        delete_remote_dir(&client, remote_root.folder_id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn remove_after() {
        // prepare basic folder
        let client = create_client();
        let remote_root = create_remote_dir(&client, 0).await.unwrap();
        let remote_file_first = create_remote_file(&client, remote_root.folder_id)
            .await
            .unwrap();
        let remote_folder_first = create_remote_dir(&client, remote_root.folder_id)
            .await
            .unwrap();
        let remote_file_second = create_remote_file(&client, remote_folder_first.folder_id)
            .await
            .unwrap();
        //
        let root = create_root();
        build_cmd(root.path(), true, vec![])
            .execute(client.clone(), remote_root.folder_id)
            .await;
        assert!(root
            .path()
            .join(remote_file_first.base.name.as_str())
            .exists());
        assert!(root
            .path()
            .join(remote_folder_first.base.name.as_str())
            .exists());
        assert!(root
            .path()
            .join(remote_folder_first.base.name.as_str())
            .join(remote_file_second.base.name.as_str())
            .exists());
        //
        let error = scan_remote_dir(&client, remote_folder_first.folder_id)
            .await
            .unwrap_err();
        assert!(matches!(error, pcloud::error::Error::Protocol(2005, _)));
    }
}
