use super::common::{get_checksum, CompareMethod};
use async_recursion::async_recursion;
use clap::Parser;
use futures_util::future::try_join_all;
use pcloud::entry::File;
use pcloud::error::Error as PCloudError;
use pcloud::file::delete::FileDeleteCommand;
use pcloud::file::download::FileDownloadCommand;
use pcloud::file::get_info::FileCheckSumCommand;
use pcloud::folder::delete::FolderDeleteCommand;
use pcloud::folder::list::FolderListCommand;
use pcloud::http::HttpClient;
use pcloud::prelude::HttpCommand;
use std::fs;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};

#[derive(Debug)]
enum Error {
    PCloud(PCloudError),
    Io(IoError),
    Retry(Vec<Error>),
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

async fn should_download_file_with_checksum(
    pcloud: &HttpClient,
    remote_file: &File,
    local_path: &Path,
) -> Result<bool, Error> {
    if local_path.exists() {
        match get_checksum(local_path) {
            Ok(checksum) => {
                let remote_checksum = FileCheckSumCommand::new(remote_file.file_id.into())
                    .execute(pcloud)
                    .await?;
                if remote_checksum.sha256 != checksum {
                    tracing::debug!("{:?} checksum mismatch, downloading again", local_path);
                }
                Ok(remote_checksum.sha256 != checksum)
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
        remote_file: &File,
        local_path: &Path,
    ) -> Result<bool, Error> {
        match self {
            Self::Checksum => {
                should_download_file_with_checksum(pcloud, remote_file, local_path).await
            }
            Self::Force => Ok(true),
            Self::Presence => Ok(!local_path.exists()),
        }
    }
}

#[derive(Parser)]
pub struct Command {
    /// Remote remove files when downloaded.
    #[clap(long)]
    remove_after_download: bool,
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
    fn should_exclude_file(&self, fpath: &Path) -> bool {
        self.exclude.iter().any(|p| p.matches_path(fpath))
    }

    async fn handle_file(
        &self,
        pcloud: &HttpClient,
        remote_path: &Path,
        remote_file: &File,
        local_path: &Path,
    ) -> Result<(), Error> {
        if self.should_exclude_file(remote_path) {
            tracing::info!(
                "{:?} matches one of the excluding rules, skipping",
                remote_path
            );
            return Ok(());
        }
        if self
            .compare_method
            .should_download_file(pcloud, remote_file, local_path)
            .await?
        {
            tracing::info!("{:?} downloading to {:?}", remote_path, local_path);
            let file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&local_path)?;
            FileDownloadCommand::new(remote_file.file_id.into(), file)
                .execute(pcloud)
                .await?;
            tracing::info!("{:?} downloaded successfully", remote_path);
        }
        if self.remove_after_download {
            tracing::info!("{:?} deleting file", remote_path);
            FileDeleteCommand::new(remote_file.file_id.into())
                .execute(pcloud)
                .await?;
        }
        Ok(())
    }

    async fn handle_file_with_retry(
        &self,
        pcloud: &HttpClient,
        remote_path: PathBuf,
        remote_file: &File,
        local_path: PathBuf,
    ) -> Result<(), Error> {
        let mut errors = Vec::with_capacity(self.retries);
        for count in 0..self.retries {
            tracing::info!("{:?} download file, try {}", remote_path, count);
            match self
                .handle_file(pcloud, &remote_path, remote_file, &local_path)
                .await
            {
                Ok(_) => return Ok(()),
                Err(err) => {
                    tracing::warn!("{:?} unable to download file: {:?}", remote_path, err);
                    errors.push(err);
                }
            }
        }
        Err(Error::Retry(errors))
    }

    #[async_recursion(?Send)]
    async fn handle_folder(
        &self,
        pcloud: &HttpClient,
        remote_path: &Path,
        folder_id: u64,
        local_path: &Path,
    ) -> Result<(), Error> {
        tracing::info!("{:?} downloading folder to {:?}", remote_path, local_path);
        let remote_folder = FolderListCommand::new(folder_id.into())
            .execute(pcloud)
            .await?;
        if let Some(ref contents) = remote_folder.contents {
            let files = contents
                .iter()
                .filter_map(|entry| entry.as_file())
                .map(|file| {
                    let path = remote_path.join(file.base.name.as_str());
                    let local_file = local_path.join(file.base.name.as_str());
                    self.handle_file_with_retry(pcloud, path, file, local_file)
                })
                .collect::<Vec<_>>();
            try_join_all(files).await?;

            let folders = contents
                .iter()
                .filter_map(|entry| entry.as_folder())
                .filter_map(|folder| {
                    let path = remote_path.join(folder.base.name.as_str());
                    let local_folder = local_path.join(folder.base.name.as_str());
                    if let Err(err) = fs::create_dir_all(&local_folder) {
                        tracing::warn!("unable to create folder {:?}: {:?}", local_folder, err);
                        None
                    } else {
                        Some((path, folder.folder_id, local_folder))
                    }
                })
                .map(|(path, fid, local_folder)| {
                    self.handle_folder_with_retry(pcloud, path, fid, local_folder)
                })
                .collect::<Vec<_>>();
            try_join_all(folders).await?;
        }
        if folder_id != 0 && self.remove_after_download {
            tracing::info!("{:?} deleting folder", remote_path);
            FolderDeleteCommand::new(folder_id.into())
                .execute(pcloud)
                .await?;
        }
        Ok(())
    }

    async fn handle_folder_with_retry(
        &self,
        pcloud: &HttpClient,
        remote_path: PathBuf,
        folder_id: u64,
        local_path: PathBuf,
    ) -> Result<(), Error> {
        let mut errors = Vec::with_capacity(self.retries);
        for count in 0..self.retries {
            tracing::info!("{:?} download folder, try {}", remote_path, count);
            match self
                .handle_folder(pcloud, &remote_path, folder_id, &local_path)
                .await
            {
                Ok(_) => return Ok(()),
                Err(err) => {
                    tracing::warn!("{:?} unable to download folder: {:?}", remote_path, err);
                    errors.push(err);
                }
            }
        }
        Err(Error::Retry(errors))
    }

    #[tracing::instrument(skip_all, level = "info")]
    pub async fn execute(&self, pcloud: HttpClient, folder_id: u64) {
        let remote_path = PathBuf::from("/");
        self.handle_folder_with_retry(&pcloud, remote_path, folder_id, self.path.clone())
            .await
            .expect("couldn't sync folder");
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
