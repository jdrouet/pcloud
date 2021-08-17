use async_recursion::async_recursion;
use clap::Clap;
use pcloud::entry::Entry;
use pcloud::http::PCloudApi;
use std::collections::HashMap;
use std::fs;
use std::path::{Path, PathBuf};

#[derive(Clap)]
pub struct Command {
    path: PathBuf,
    #[clap(long)]
    recursive: bool,
    #[clap(long)]
    only_upload: bool,
    #[clap(long)]
    only_download: bool,
    #[clap(long)]
    remove_local: bool,
    #[clap(long)]
    remove_remote: bool,
}

fn err_string<T: ToString>(err: T) -> String {
    err.to_string()
}

fn list_local_entries(folder: &Path) -> Result<HashMap<String, PathBuf>, String> {
    let entries = std::fs::read_dir(folder)
        .map_err(err_string)?
        .filter_map(|entry| entry.ok())
        .map(|entry| entry.path())
        .filter_map(|path| {
            path.file_name()
                .and_then(|name| name.to_str())
                .map(|name| (name.to_string(), path.clone()))
        })
        .collect();
    Ok(entries)
}

async fn list_remote_entries(
    pcloud: &PCloudApi,
    folder_id: usize,
) -> Result<HashMap<String, Entry>, String> {
    pcloud
        .list_folder(folder_id)
        .await
        .map(|res| res.contents.unwrap_or_default())
        .map(|res| {
            res.into_iter()
                .map(|entry| (entry.base().name.clone(), entry))
                .collect::<HashMap<_, _>>()
        })
        .map_err(|err| err.to_string())
}

impl Command {
    fn should_upload(&self) -> bool {
        !self.only_download
    }

    fn should_download(&self) -> bool {
        !self.only_upload
    }

    async fn upload_file(
        &self,
        pcloud: &PCloudApi,
        fname: &str,
        fpath: &Path,
        folder_id: usize,
    ) -> Result<(), String> {
        println!("uploading file {:?} to folder {}", fpath, folder_id);
        let file = fs::File::open(&fpath).map_err(err_string)?;
        pcloud
            .upload_file(file, &fname, folder_id)
            .await
            .map_err(err_string)?;
        if self.remove_local {
            fs::remove_file(&fpath).map_err(err_string)?;
        }
        Ok(())
    }

    async fn create_folder(
        &self,
        pcloud: &PCloudApi,
        fname: &str,
        fpath: &Path,
        folder_id: usize,
    ) -> Result<(), String> {
        let folder = pcloud
            .create_folder(fname, folder_id)
            .await
            .map_err(err_string)?;
        self.sync_folder(pcloud, folder.folder_id, fpath).await?;
        Ok(())
    }

    #[async_recursion]
    async fn sync_folder(
        &self,
        pcloud: &PCloudApi,
        folder_id: usize,
        path: &Path,
    ) -> Result<(), String> {
        println!(
            "synchronize local folder {:?} with folder {}",
            path, folder_id
        );
        let local_entries = list_local_entries(&path)?;
        let remote_entries = list_remote_entries(&pcloud, folder_id).await?;

        if self.should_upload() {
            let remote_missing = local_entries
                .iter()
                .filter(|(fname, _)| !remote_entries.contains_key(*fname));

            for (fname, fpath) in remote_missing {
                if fpath.is_file() {
                    self.upload_file(pcloud, fname, fpath, folder_id).await?;
                } else if fpath.is_dir() && self.recursive {
                    self.create_folder(pcloud, fname, fpath, folder_id).await?;
                }
            }
        }

        if self.should_download() {
            // TODO
            let _local_missing = remote_entries
                .iter()
                .filter(|(fname, _)| !local_entries.contains_key(*fname));
        }

        if self.recursive {
            for (fid, path) in local_entries
                .iter()
                .filter(|(_, fpath)| fpath.is_dir())
                .filter_map(|(fname, fpath)| {
                    remote_entries
                        .get(fname)
                        .and_then(|entry| entry.folder_id())
                        .map(|folder_id| (folder_id, fpath))
                })
            {
                self.sync_folder(pcloud, fid, path).await?;
            }
        }

        Ok(())
    }

    pub async fn execute(&self, pcloud: PCloudApi, folder_id: usize) {
        self.sync_folder(&pcloud, folder_id, &self.path)
            .await
            .expect("couldn't synchronize")
    }
}
