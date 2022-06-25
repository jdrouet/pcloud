use async_recursion::async_recursion;
use clap::Parser;
use pcloud::entry::{Entry, File, Folder};
use pcloud::error::Error as PCloudError;
use pcloud::file::delete::FileDeleteCommand;
use pcloud::file::download::FileDownloadCommand;
use pcloud::file::upload::FileUploadCommand;
use pcloud::folder::create::FolderCreateCommand;
use pcloud::folder::delete::FolderDeleteCommand;
use pcloud::folder::list::FolderListCommand;
use pcloud::http::HttpClient;
use pcloud::prelude::HttpCommand;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::Error as IoError;
use std::path::{Path, PathBuf};

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

enum LocalEntry {
    File(PathBuf),
    Folder(PathBuf),
}

impl LocalEntry {
    fn path(&self) -> &Path {
        match self {
            Self::File(path) => path,
            Self::Folder(path) => path,
        }
    }

    fn name(&self) -> Option<String> {
        self.path()
            .file_name()
            .and_then(|name| name.to_str())
            .map(String::from)
    }
}

fn get_remote_entries(folder: &Folder) -> HashMap<String, Entry> {
    folder
        .contents
        .as_ref()
        .map(|list| {
            list.iter()
                .map(|entry| (entry.base().name.clone(), entry.clone()))
                .collect()
        })
        .unwrap_or_default()
}

fn get_local_entries(path: &Path) -> Result<HashMap<String, LocalEntry>, Error> {
    let result = fs::read_dir(path)?
        .filter_map(|entry| entry.ok())
        .map(|item| item.path())
        .filter_map(|item| {
            if item.is_file() {
                Some(LocalEntry::File(item))
            } else if item.is_dir() {
                Some(LocalEntry::Folder(item))
            } else {
                None
            }
        })
        .filter_map(|item| item.name().map(|name| (name, item)))
        .collect();
    Ok(result)
}

#[derive(Parser)]
pub struct Command {
    /// Disable uploading local files.
    #[clap(long)]
    disable_upload: bool,
    /// Remove local files when uploaded.
    #[clap(long)]
    remove_after_upload: bool,
    /// Disable downloading remote files.
    #[clap(long)]
    disable_download: bool,
    /// Remote remove files when downloaded.
    #[clap(long)]
    remove_after_download: bool,
    /// Keep partial file if upload fails.
    #[clap(long)]
    allow_partial_upload: bool,
    /// Number of allowed failure.
    #[clap(long, default_value_t = 5)]
    retries: usize,
    /// Local folder to synchronize.
    #[clap()]
    path: PathBuf,
}

impl Command {
    async fn download_file(
        &self,
        pcloud: &HttpClient,
        remote_name: &str,
        remote_file: &File,
        local_path: &Path,
    ) -> Result<(), Error> {
        tracing::info!("downloading {} to {:?}", remote_name, local_path);
        let path = local_path.join(remote_name);
        let file = fs::File::create(&path)?;
        FileDownloadCommand::new(remote_file.file_id.into(), file)
            .execute(pcloud)
            .await?;
        tracing::info!("downloaded {} successfully", remote_name);
        if self.remove_after_download {
            tracing::info!("deleting {}", remote_name);
            FileDeleteCommand::new(remote_file.file_id.into())
                .execute(pcloud)
                .await?;
        }
        Ok(())
    }

    async fn download_folder(
        &self,
        pcloud: &HttpClient,
        remote_name: &str,
        remote_folder: &Folder,
        local_path: &Path,
    ) -> Result<(), Error> {
        tracing::info!("downloading folder {} to {:?}", remote_name, local_path);
        let local_folder = local_path.join(remote_name);
        fs::create_dir(&local_folder)?;
        self.sync_folder(pcloud, remote_folder, &local_folder)
            .await?;
        tracing::info!("downloaded folder {:?}", local_folder);
        if self.remove_after_download {
            tracing::info!("deleting folder {}", remote_name);
            FolderDeleteCommand::new(remote_folder.folder_id.into())
                .execute(pcloud)
                .await?;
        }
        Ok(())
    }

    async fn sync_remote_entries(
        &self,
        pcloud: &HttpClient,
        remote_names: impl Iterator<Item = &String>,
        remote_entries: &HashMap<String, Entry>,
        path: &Path,
    ) -> Result<(), Error> {
        for remote_name in remote_names {
            match remote_entries.get(remote_name) {
                Some(Entry::File(remote_file)) => {
                    self.download_file(pcloud, remote_name, remote_file, path)
                        .await?;
                }
                Some(Entry::Folder(remote_folder)) => {
                    self.download_folder(pcloud, remote_name, remote_folder, path)
                        .await?;
                }
                None => {}
            }
        }
        Ok(())
    }

    async fn upload_file(
        &self,
        pcloud: &HttpClient,
        local_name: &str,
        local_file: &Path,
        remote_folder: &Folder,
    ) -> Result<(), Error> {
        let file = fs::File::open(local_file)?;
        FileUploadCommand::new(local_name, remote_folder.folder_id, file)
            .no_partial(!self.allow_partial_upload)
            .execute(pcloud)
            .await?;
        tracing::info!("uploaded {:?}", local_file);
        if self.remove_after_upload {
            tracing::info!("deleting file {:?}", local_file);
            if let Err(error) = fs::remove_file(&local_file) {
                tracing::warn!("unable to delete file {:?}: {:?}", local_file, error);
            }
        }
        Ok(())
    }

    #[async_recursion(?Send)]
    async fn upload_file_with_retry(
        &self,
        pcloud: &HttpClient,
        local_name: &str,
        local_file: &Path,
        remote_folder: &Folder,
        retry: usize,
    ) -> Result<(), Error> {
        tracing::info!(
            "uploading {:?} to {} (retry: {})",
            local_file,
            remote_folder.folder_id,
            retry
        );
        if let Err(err) = self
            .upload_file(pcloud, local_name, local_file, remote_folder)
            .await
        {
            if retry > 0 {
                self.upload_file_with_retry(
                    pcloud,
                    local_name,
                    local_file,
                    remote_folder,
                    retry - 1,
                )
                .await
            } else {
                Err(err)
            }
        } else {
            Ok(())
        }
    }

    async fn upload_folder(
        &self,
        pcloud: &HttpClient,
        local_name: &str,
        local_folder: &Path,
        remote_folder: &Folder,
    ) -> Result<(), Error> {
        tracing::info!(
            "uploading folder {:?} to {}",
            local_folder,
            remote_folder.folder_id
        );
        let created = FolderCreateCommand::new(local_name.to_string(), remote_folder.folder_id)
            .ignore_exists(true)
            .execute(pcloud)
            .await?;
        tracing::info!("uploaded folder {:?}", local_folder);
        self.sync_folder(pcloud, &created, local_folder).await?;
        if self.remove_after_upload {
            tracing::info!("deleting folder {:?}", local_folder);
            fs::remove_dir_all(local_folder)?;
        }
        Ok(())
    }

    async fn sync_local_entries(
        &self,
        pcloud: &HttpClient,
        local_names: impl Iterator<Item = &String>,
        local_entries: &HashMap<String, LocalEntry>,
        folder: &Folder,
    ) -> Result<(), Error> {
        for local_name in local_names {
            match local_entries.get(local_name) {
                Some(LocalEntry::File(path)) => {
                    self.upload_file_with_retry(pcloud, local_name, path, folder, self.retries)
                        .await?;
                }
                Some(LocalEntry::Folder(path)) => {
                    self.upload_folder(pcloud, local_name, path, folder).await?;
                }
                None => {}
            }
        }
        Ok(())
    }

    async fn sync_common_entries(
        &self,
        pcloud: &HttpClient,
        common_names: impl Iterator<Item = &String>,
        remote_entries: &HashMap<String, Entry>,
        local_entries: &HashMap<String, LocalEntry>,
        _folder: &Folder,
        _path: &Path,
    ) -> Result<(), Error> {
        for (_name, remote, local) in common_names.filter_map(|name| {
            remote_entries
                .get(name)
                .and_then(|remote| local_entries.get(name).map(|local| (name, remote, local)))
        }) {
            if let Entry::Folder(remote_folder) = remote {
                self.sync_folder(pcloud, remote_folder, local.path())
                    .await?;
            }
        }
        Ok(())
    }

    #[async_recursion(?Send)]
    async fn sync_folder(
        &self,
        pcloud: &HttpClient,
        remote_folder: &Folder,
        local_path: &Path,
    ) -> Result<(), Error> {
        let remote_entries = get_remote_entries(remote_folder);
        let local_entries = get_local_entries(local_path)?;

        let remote_names: HashSet<String> = remote_entries.keys().cloned().collect();
        let local_names: HashSet<String> = local_entries.keys().cloned().collect();

        if !self.disable_download {
            let only_remote = remote_names.difference(&local_names);
            self.sync_remote_entries(pcloud, only_remote, &remote_entries, local_path)
                .await?;
        }

        if !self.disable_upload {
            let only_local = local_names.difference(&remote_names);
            self.sync_local_entries(pcloud, only_local, &local_entries, remote_folder)
                .await?;
        }

        let common_names = remote_names.intersection(&local_names);
        self.sync_common_entries(
            pcloud,
            common_names,
            &remote_entries,
            &local_entries,
            remote_folder,
            local_path,
        )
        .await?;

        Ok(())
    }

    #[tracing::instrument(skip_all, level = "info")]
    pub async fn execute(&self, pcloud: HttpClient, folder_id: u64) {
        let remote_folder = FolderListCommand::new(folder_id.into())
            .execute(&pcloud)
            .await
            .expect("unable to get folder");
        self.sync_folder(&pcloud, &remote_folder, &self.path)
            .await
            .expect("couldn't sync folder");
    }
}
