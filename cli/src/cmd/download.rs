use std::collections::VecDeque;
use std::io::Read;
use std::path::PathBuf;

use pcloud::client::HttpClient;
use pcloud::entry::{Entry, File, Folder};
use pcloud::file::FileIdentifier;
use pcloud::prelude::HttpCommand;

fn compute_sha1(path: &PathBuf) -> std::io::Result<String> {
    let mut file = std::fs::OpenOptions::new().read(true).open(path)?;
    let mut buffer = [0u8; 4096];
    let mut hasher = sha1_smol::Sha1::new();
    loop {
        let size = file.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        hasher.update(&buffer[..size]);
    }
    Ok(hasher.digest().to_string())
}

struct DownloadManager<'a> {
    client: &'a HttpClient,
    dry_run: bool,
    skip_existing: bool,
    queue: VecDeque<(Entry, PathBuf)>,
}

impl<'a> DownloadManager<'a> {
    fn new(client: &'a HttpClient, dry_run: bool, skip_existing: bool) -> Self {
        Self {
            client,
            dry_run,
            skip_existing,
            queue: VecDeque::with_capacity(1),
        }
    }

    async fn process_file(&mut self, file: File, target: PathBuf) -> anyhow::Result<usize> {
        tracing::info!("downloading file {:?} to {target:?}", file.base.name);
        if self.dry_run {
            return Ok(0);
        }
        if target.exists() {
            if self.skip_existing {
                tracing::info!("file already exists, skipping...");
                return Ok(0);
            }
            tracing::info!("computing existing file hash...");
            let fhash = compute_sha1(&target)?;
            let ident = FileIdentifier::file_id(file.file_id);
            let checksum = pcloud::file::checksum::FileCheckSumCommand::new(ident)
                .execute(&self.client)
                .await?;
            if fhash == checksum.sha1 {
                tracing::info!("file already exists with same hash, skipping...");
                return Ok(0);
            }
            tracing::info!("file hashes differ {fhash} != {}", checksum.sha1);
        }
        if let Some(parent) = target.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(&parent)?;
            }
        }
        let writer = std::fs::OpenOptions::new()
            .write(true)
            .create(true)
            .open(target)?;
        let ident = FileIdentifier::file_id(file.file_id);
        let count = pcloud::file::download::FileDownloadCommand::new(ident, writer)
            .execute(self.client)
            .await?;
        Ok(count)
    }

    async fn process_folder(&mut self, folder: Folder, target: PathBuf) -> anyhow::Result<()> {
        tracing::info!("downloading folder {:?} to {target:?}", folder.base.name);
        if let Some(children) = folder.contents {
            for child in children {
                let new_target = target.join(child.base().name.as_str());
                self.queue.push_back((child, new_target));
            }
        } else {
            let res = pcloud::folder::list::FolderListCommand::new(folder.folder_id.into())
                .execute(self.client)
                .await?;
            for child in res.contents.into_iter().flatten() {
                let new_target = target.join(child.base().name.as_str());
                self.queue.push_back((child, new_target));
            }
        }
        Ok(())
    }

    async fn run(mut self, entry: Entry, target: PathBuf) -> anyhow::Result<()> {
        let mut count = 0;
        self.queue.push_front((entry, target));
        while let Some((entry, target)) = self.queue.pop_front() {
            match entry {
                Entry::File(file) => {
                    count += self.process_file(file, target).await?;
                }
                Entry::Folder(folder) => {
                    self.process_folder(folder, target).await?;
                }
            }
            tracing::info!("{} elements left in the queue", self.queue.len());
        }
        let formatter = human_number::Formatter::binary()
            .with_decimals(2)
            .with_unit("B");
        let value = formatter.format(count as f64);
        tracing::info!("downloaded {value}");
        Ok(())
    }
}

#[derive(clap::Parser)]
pub(crate) struct Command {
    /// When enabled, nothing will be really created on disk
    #[clap(long)]
    dry_run: bool,
    /// When enabled, if the file already exists, the download is skipped.
    #[clap(long)]
    skip_existing: bool,

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

    pub(crate) async fn execute(self, client: &HttpClient) -> anyhow::Result<()> {
        let result = self.fetch_entry(client).await?;
        let manager = DownloadManager::new(client, self.dry_run, self.skip_existing);
        match result {
            Entry::File(file) => {
                let target = if self.local_path.exists() && self.local_path.is_dir() {
                    self.local_path.join(file.base.name.as_str())
                } else {
                    self.local_path.clone()
                };
                manager.run(Entry::File(file), target).await?;
            }
            Entry::Folder(folder) => {
                if self.local_path.is_file() {
                    return Err(anyhow::anyhow!("the provided local path is a file"));
                }
                manager.run(Entry::Folder(folder), self.local_path).await?;
            }
        }
        Ok(())
    }
}
