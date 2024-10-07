use std::{collections::VecDeque, path::PathBuf};

use pcloud::{client::HttpClient, prelude::HttpCommand};

enum Action {
    File {
        local: PathBuf,
        parent_id: u64,
        name: String,
    },
    Folder {
        local: PathBuf,
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
        parent_id: u64,
        fname: String,
    ) -> anyhow::Result<()> {
        tracing::info!("uploading {local:?} to {parent_id}");
        let reader = std::fs::OpenOptions::new().read(true).open(local)?;
        pcloud::file::upload::FileUploadCommand::new(fname, parent_id, reader)
            .execute(self.client)
            .await?;
        Ok(())
    }

    async fn upload_folder(&mut self, local: PathBuf, folder_id: u64) -> anyhow::Result<()> {
        for entry in std::fs::read_dir(local)? {
            let entry = entry?;
            let name = entry.file_name().to_string_lossy().to_string();
            let path = entry.path();
            if path.is_file() {
                self.queue.push_back(Action::File {
                    local: path,
                    parent_id: folder_id,
                    name,
                });
            } else if path.is_dir() {
                let folder = pcloud::folder::create::FolderCreateCommand::new(name, folder_id)
                    .with_ignore_exists(true)
                    .execute(self.client)
                    .await?;
                self.queue.push_back(Action::Folder {
                    local: path,
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
                    parent_id,
                    name,
                } => {
                    self.upload_file(local, parent_id, name).await?;
                    count += 1;
                }
                Action::Folder { local, folder_id } => {
                    self.upload_folder(local, folder_id).await?;
                }
            }
        }
        Ok(count)
    }
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
                parent_id: folder.folder_id,
                name: fname.to_string(),
            });
        }
        if self.local_path.is_dir() {
            if self.remote_path == "/" {
                manager.queue.push_back(Action::Folder {
                    local: self.local_path,
                    folder_id: pcloud::folder::ROOT,
                });
            } else {
                let remote_path = match self.remote_path.strip_suffix('/') {
                    Some(inner) => inner,
                    None => self.remote_path.as_str(),
                };
                let folder = pcloud::folder::list::FolderListCommand::new(remote_path.into())
                    .execute(client)
                    .await?;
                manager.queue.push_back(Action::Folder {
                    local: self.local_path,
                    folder_id: folder.folder_id,
                });
            }
        }

        manager.run().await?;

        Ok(())
    }
}
