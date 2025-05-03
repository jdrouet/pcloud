use std::collections::VecDeque;
use std::io::Read;
use std::path::{Path, PathBuf};

use futures::StreamExt;
use pcloud::entry::Entry;
use pcloud::file::File;
use pcloud::folder::Folder;
use pcloud::Client;
use tokio::io::{AsyncWriteExt, BufWriter};

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
    client: &'a Client,
    dry_run: bool,
    skip_existing: bool,
    queue: VecDeque<(Entry, PathBuf)>,
}

async fn download_file(url: String, path: impl AsRef<Path>) -> anyhow::Result<usize> {
    let writer = tokio::fs::OpenOptions::new()
        .write(true)
        .create(true)
        .truncate(true)
        .open(path)
        .await?;
    let mut writer = BufWriter::new(writer);

    let res = pcloud::reqwest::get(&url).await?;
    res.error_for_status_ref()?;
    let mut stream = res.bytes_stream();

    let mut size: usize = 0;
    while let Some(chunk) = stream.next().await {
        let chunk = chunk?;
        size += chunk.len();
        writer.write_all(&chunk).await?;
    }

    Ok(size)
}

impl<'a> DownloadManager<'a> {
    fn new(client: &'a Client, dry_run: bool, skip_existing: bool) -> Self {
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
            let checksum = self.client.get_file_checksum(file.file_id).await?;
            if fhash == checksum.sha1 {
                tracing::info!("file already exists with same hash, skipping...");
                return Ok(0);
            }
            tracing::info!("file hashes differ {fhash} != {}", checksum.sha1);
        }
        if let Some(parent) = target.parent() {
            if !parent.exists() {
                std::fs::create_dir_all(parent)?;
            }
        }
        let file_links = self.client.get_file_link(file.file_id).await?;
        for link in file_links.links() {
            match download_file(link.to_string(), &target).await {
                Ok(size) => return Ok(size),
                Err(err) => {
                    tracing::error!(message = "download failed", cause = %err);
                }
            }
        }
        anyhow::bail!("unable to download file")
    }

    async fn process_folder(&mut self, folder: Folder, target: PathBuf) -> anyhow::Result<()> {
        tracing::info!("downloading folder {:?} to {target:?}", folder.base.name);
        if let Some(children) = folder.contents {
            for child in children {
                let new_target = target.join(child.base().name.as_str());
                self.queue.push_back((child, new_target));
            }
        } else {
            let res = self.client.list_folder(folder.folder_id).await?;
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
    async fn fetch_entry(&self, client: &Client) -> Result<Entry, pcloud::Error> {
        // assuming it's doing a `ls` on a folder at first
        match client.list_folder(&self.remote_path).await {
            Ok(folder) => Ok(Entry::Folder(folder)),
            Err(pcloud::Error::Protocol(2005, _)) => {
                // try with a file if a folder is not found
                client
                    .get_file_checksum(&self.remote_path)
                    .await
                    .map(|res| Entry::File(res.metadata))
            }
            Err(inner) => Err(inner),
        }
    }

    pub(crate) async fn execute(self, client: &Client) -> anyhow::Result<()> {
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
