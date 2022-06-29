use super::common::{get_checksum, CompareMethod};
use async_recursion::async_recursion;
use clap::Parser;
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
    /// Number of allowed failure.
    #[clap(long, default_value_t = 5)]
    retries: usize,
    /// Local folder to synchronize.
    #[clap()]
    path: PathBuf,
}

impl Command {
    async fn handle_file(
        &self,
        pcloud: &HttpClient,
        remote_file: &File,
        local_path: &Path,
    ) -> Result<(), Error> {
        tracing::info!("downloading {} to {:?}", remote_file.base.name, local_path);
        if self
            .compare_method
            .should_download_file(pcloud, remote_file, local_path)
            .await?
        {
            let file = fs::OpenOptions::new()
                .create(true)
                .write(true)
                .truncate(true)
                .open(&local_path)?;
            FileDownloadCommand::new(remote_file.file_id.into(), file)
                .execute(pcloud)
                .await?;
            tracing::info!("downloaded {} successfully", remote_file.base.name);
        }
        if self.remove_after_download {
            tracing::info!("deleting file {}", remote_file.base.name);
            FileDeleteCommand::new(remote_file.file_id.into())
                .execute(pcloud)
                .await?;
        }
        Ok(())
    }

    async fn handle_file_with_retry(
        &self,
        pcloud: &HttpClient,
        remote_file: &File,
        local_path: &Path,
    ) -> Result<(), Error> {
        let mut errors = Vec::with_capacity(self.retries);
        for count in 0..self.retries {
            tracing::info!("download file {:?}, try {}", local_path, count);
            match self.handle_file(pcloud, remote_file, local_path).await {
                Ok(_) => return Ok(()),
                Err(err) => {
                    tracing::warn!("unable to download file {:?}: {:?}", local_path, err);
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
        folder_id: u64,
        local_path: &Path,
    ) -> Result<(), Error> {
        tracing::info!("downloading folder {} to {:?}", folder_id, local_path);
        let remote_folder = FolderListCommand::new(folder_id.into())
            .execute(pcloud)
            .await?;
        if let Some(ref contents) = remote_folder.contents {
            for file in contents.iter().filter_map(|entry| entry.as_file()) {
                let local_file = local_path.join(file.base.name.as_str());
                self.handle_file_with_retry(pcloud, file, &local_file)
                    .await?;
            }
            for folder in contents.iter().filter_map(|entry| entry.as_folder()) {
                let local_folder = local_path.join(folder.base.name.as_str());
                fs::create_dir_all(&local_folder)?;
                self.handle_folder_with_retry(pcloud, folder.folder_id, &local_folder)
                    .await?;
            }
        }
        if folder_id != 0 && self.remove_after_download {
            tracing::info!("deleting folder {}", folder_id);
            FolderDeleteCommand::new(folder_id.into())
                .execute(pcloud)
                .await?;
        }
        Ok(())
    }

    async fn handle_folder_with_retry(
        &self,
        pcloud: &HttpClient,
        folder_id: u64,
        local_path: &Path,
    ) -> Result<(), Error> {
        let mut errors = Vec::with_capacity(self.retries);
        for count in 0..self.retries {
            tracing::info!("download folder {:?}, try {}", folder_id, count);
            match self.handle_folder(pcloud, folder_id, local_path).await {
                Ok(_) => return Ok(()),
                Err(err) => {
                    tracing::warn!("unable to download folder {:?}: {:?}", folder_id, err);
                    errors.push(err);
                }
            }
        }
        Err(Error::Retry(errors))
    }

    #[tracing::instrument(skip_all, level = "info")]
    pub async fn execute(&self, pcloud: HttpClient, folder_id: u64) {
        self.handle_folder_with_retry(&pcloud, folder_id, &self.path)
            .await
            .expect("couldn't sync folder");
    }
}

#[cfg(all(test, feature = "protected"))]
mod tests {
    use super::Command;
    use crate::tests::*;
    use std::path::{Path, PathBuf};

    fn build_cmd(root: &Path, remove_after_download: bool) -> Command {
        Command {
            remove_after_download,
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
        build_cmd(root.path(), false)
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
        build_cmd(root.path(), false)
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
        build_cmd(root.path(), true)
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
