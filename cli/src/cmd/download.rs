use std::path::{Path, PathBuf};

use pcloud::client::HttpClient;
use pcloud::entry::{Entry, File, Folder};
use pcloud::file::FileIdentifier;
use pcloud::prelude::HttpCommand;

fn is_empty(path: &Path) -> std::io::Result<bool> {
    let mut content = std::fs::read_dir(path)?;
    Ok(content.next().is_none())
}

fn collect_files(folder: &Folder, path: PathBuf) -> Vec<(&File, PathBuf)> {
    let mut queue = vec![(folder, path)];
    let mut res = Vec::new();
    while let Some((folder, path)) = queue.pop() {
        for item in folder.contents.iter().flat_map(|item| item.iter()) {
            match item {
                Entry::Folder(inner) => {
                    queue.push((inner, path.join(inner.base.name.as_str())));
                }
                Entry::File(inner) => {
                    res.push((inner, path.join(inner.base.name.as_str())));
                }
            }
        }
    }
    res
}

#[derive(clap::Parser)]
pub(crate) struct Command {
    /// Remote path to the file or directory to download
    remote_path: String,
    /// Local path to download the file or directory to
    local_path: PathBuf,
}

impl Command {
    async fn fetch_entry(&self, client: &HttpClient) -> Result<Entry, pcloud::error::Error> {
        // assuming it's doing a `ls` on a folder at first
        let folder_id = pcloud::folder::FolderIdentifier::path(&self.remote_path);
        let folder_res = pcloud::folder::list::FolderListCommand::new(folder_id)
            .with_recursive(true)
            .execute(client)
            .await;
        match folder_res {
            Ok(folder) => Ok(Entry::Folder(folder)),
            Err(pcloud::error::Error::Protocol(2005, _)) => {
                // try with a file if a folder is not found
                let file_id = pcloud::file::FileIdentifier::path(&self.remote_path);
                pcloud::file::checksum::FileCheckSumCommand::new(file_id)
                    .execute(client)
                    .await
                    .map(|res| Entry::File(res.metadata))
            }
            Err(inner) => Err(inner),
        }
    }

    async fn download_file(
        &self,
        client: &HttpClient,
        file: &File,
        target: PathBuf,
    ) -> anyhow::Result<()> {
        let writer = std::fs::OpenOptions::new()
            .write(true)
            .create_new(true)
            .open(target)?;
        let ident = FileIdentifier::file_id(file.file_id);
        pcloud::file::download::FileDownloadCommand::new(ident, writer)
            .execute(client)
            .await?;
        Ok(())
    }

    async fn download_folder(
        &self,
        client: &HttpClient,
        folder: &Folder,
        target: PathBuf,
    ) -> anyhow::Result<()> {
        let files = collect_files(folder, target);
        for (file, path) in files.into_iter() {
            if let Some(parent) = path.parent() {
                if !parent.exists() {
                    std::fs::create_dir_all(parent)?;
                }
            }
            self.download_file(client, file, path).await?;
        }
        Ok(())
    }

    pub(crate) async fn execute(self, client: &HttpClient) -> anyhow::Result<()> {
        let result = self.fetch_entry(client).await?;
        match result {
            Entry::File(file) => {
                let target = if self.local_path.exists() && self.local_path.is_dir() {
                    self.local_path.join(file.base.name.as_str())
                } else if self.local_path.exists() && self.local_path.is_file() {
                    return Err(anyhow::anyhow!("the target file already exists"));
                } else {
                    self.local_path.clone()
                };
                self.download_file(client, &file, target).await?;
            }
            Entry::Folder(folder) => {
                if self.local_path.is_file() {
                    return Err(anyhow::anyhow!("the provided local path is a file"));
                }
                let local = if self.local_path.exists() && !is_empty(&self.local_path)? {
                    self.local_path.join(folder.base.name.as_str())
                } else {
                    self.local_path.clone()
                };
                self.download_folder(client, &folder, local).await?;
            }
        }
        Ok(())
    }
}
