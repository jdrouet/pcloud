use std::{collections::VecDeque, path::PathBuf};

use pcloud::{client::HttpClient, prelude::HttpCommand};

use crate::helper::{
    file::compute_sha1,
    remote::{get_folder, maybe_get_file, maybe_get_folder},
};

enum Action {
    File {
        local: PathBuf,
        parent_path: String,
        parent_id: u64,
        name: String,
    },
    Folder {
        local: PathBuf,
        remote_path: String,
        folder_id: u64,
    },
}

struct UploadManager<'a> {
    client: &'a HttpClient,
    queue: VecDeque<Action>,
}

impl<'a> UploadManager<'a> {
    async fn upload_file(
        &mut self,
        local: PathBuf,
        parent_path: String,
        parent_id: u64,
        fname: String,
    ) -> anyhow::Result<()> {
        tracing::info!("uploading {local:?} to {parent_path}/{fname}");
        let rpath = format!("{parent_path}/{fname}");
        if let Some((_, rsha)) = maybe_get_file(self.client, &rpath).await? {
            let lsha = compute_sha1(&local)?;
            if rsha == lsha {
                tracing::info!("{local:?} already exist online with the same hash, skipping...");
                return Ok(());
            }
        }
        let reader = std::fs::OpenOptions::new().read(true).open(local)?;
        pcloud::file::upload::FileUploadCommand::new(fname, parent_id, reader)
            .execute(self.client)
            .await?;
        Ok(())
    }

    async fn upload_folder(
        &mut self,
        local: PathBuf,
        remote_path: String,
        folder_id: u64,
    ) -> anyhow::Result<()> {
        tracing::info!("uploading {local:?} content to {remote_path}");
        for entry in std::fs::read_dir(local)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();
            if path.is_file() {
                self.queue.push_back(Action::File {
                    local: path,
                    parent_path: remote_path.clone(),
                    parent_id: folder_id,
                    name,
                });
            } else if path.is_dir() {
                let folder =
                    pcloud::folder::create::FolderCreateCommand::new(name.as_str(), folder_id)
                        .with_ignore_exists(true)
                        .execute(self.client)
                        .await?;
                self.queue.push_back(Action::Folder {
                    local: path,
                    remote_path: format!("{remote_path}/{name}"),
                    folder_id: folder.folder_id,
                });
            }
        }
        Ok(())
    }

    async fn run(mut self) -> anyhow::Result<usize> {
        let mut count = 0;
        while let Some(action) = self.queue.pop_front() {
            match action {
                Action::File {
                    local,
                    parent_path,
                    parent_id,
                    name,
                } => {
                    self.upload_file(local, parent_path, parent_id, name)
                        .await?;
                    count += 1;
                }
                Action::Folder {
                    local,
                    remote_path,
                    folder_id,
                } => {
                    self.upload_folder(local, remote_path, folder_id).await?;
                }
            }
        }
        Ok(count)
    }
}

async fn find_or_create_with_parent(
    client: &HttpClient,
    path: &str,
) -> Result<u64, pcloud::error::Error> {
    if let Some(folder) = maybe_get_folder(client, path).await? {
        return Ok(folder.folder_id);
    }
    tracing::info!("provided folder not found, creating...");
    // this should always happend, considering we make sure it starts with '/'
    let (parent, name) = path.rsplit_once('/').unwrap();
    let parent_id = if parent.is_empty() {
        pcloud::folder::ROOT
    } else {
        get_folder(client, parent).await.map(|f| f.folder_id)?
    };
    pcloud::folder::create::FolderCreateCommand::new(name, parent_id)
        .execute(client)
        .await
        .map(|f| f.folder_id)
}

#[derive(clap::Parser)]
pub(crate) struct Command {
    /// When enabled, nothing will be really created on disk
    #[clap(long)]
    dry_run: bool,

    /// Local file or directory to upload
    local_path: PathBuf,
    /// Remote path to upload to
    remote_path: String,
}

impl Command {
    pub(crate) async fn execute(self, client: &HttpClient) -> anyhow::Result<()> {
        if !self.local_path.exists() {
            return Err(anyhow::anyhow!(
                "provided local path {:?} does not exist",
                self.local_path
            ));
        }
        if !self.remote_path.starts_with('/') {
            return Err(anyhow::anyhow!(
                "the provided remote path should be absolute"
            ));
        }

        let mut manager = UploadManager {
            client,
            queue: Default::default(),
        };

        if self.local_path.is_file() {
            let (parent, fname) = if self.remote_path.ends_with('/') {
                let parent = self.remote_path.as_str();
                let Some(fname) = self.local_path.file_name().and_then(|v| v.to_str()) else {
                    return Err(anyhow::anyhow!("unable to get source filename"));
                };
                (parent, fname)
            } else {
                let (parent, fname) = self.remote_path.rsplit_once('/').unwrap();
                if parent.is_empty() {
                    ("/", fname)
                } else {
                    (parent, fname)
                }
            };
            let folder = pcloud::folder::list::FolderListCommand::new(parent.into())
                .execute(client)
                .await?;
            manager.queue.push_back(Action::File {
                local: self.local_path.clone(),
                parent_path: String::from(parent),
                parent_id: folder.folder_id,
                name: fname.to_string(),
            });
        }
        if self.local_path.is_dir() {
            if self.remote_path == "/" {
                manager.queue.push_back(Action::Folder {
                    local: self.local_path,
                    remote_path: String::from("/"),
                    folder_id: pcloud::folder::ROOT,
                });
            } else {
                let remote_path = match self.remote_path.strip_suffix('/') {
                    Some(inner) => inner,
                    None => self.remote_path.as_str(),
                };
                let folder_id = find_or_create_with_parent(client, remote_path).await?;
                manager.queue.push_back(Action::Folder {
                    local: self.local_path,
                    remote_path: String::from(remote_path),
                    folder_id,
                });
            }
        }

        manager.run().await?;

        Ok(())
    }
}
