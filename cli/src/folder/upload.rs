use super::common::get_checksum;
use async_recursion::async_recursion;
use clap::Parser;
use pcloud::entry::{Entry, Folder};
use pcloud::error::Error as PCloudError;
use pcloud::file::get_info::FileCheckSumCommand;
use pcloud::file::upload::FileUploadCommand;
use pcloud::folder::create::FolderCreateCommand;
use pcloud::folder::list::FolderListCommand;
use pcloud::http::HttpClient;
use pcloud::prelude::HttpCommand;
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

#[derive(Parser)]
pub struct Command {
    /// Remove local files when uploaded.
    #[clap(long)]
    remove_after_upload: bool,
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
    async fn should_upload_file(
        &self,
        pcloud: &HttpClient,
        fpath: &Path,
        fname: &str,
        folder: &Folder,
    ) -> Result<bool, Error> {
        if let Some(Entry::File(existing)) = folder
            .contents
            .as_ref()
            .and_then(|contents| contents.iter().find(|item| item.base().name == fname))
        {
            tracing::debug!("{:?} already exists remotely", fpath);
            match get_checksum(fpath) {
                Ok(checksum) => {
                    let remote_checksum = FileCheckSumCommand::new(existing.file_id.into())
                        .execute(pcloud)
                        .await?;
                    Ok(remote_checksum.sha256 != checksum)
                }
                Err(error) => {
                    tracing::warn!("skipping upload, {}", error);
                    Ok(false)
                }
            }
        } else {
            Ok(true)
        }
    }

    async fn handle_file(
        &self,
        pcloud: &HttpClient,
        fpath: &Path,
        fname: &str,
        folder: &Folder,
    ) -> Result<(), Error> {
        if self
            .should_upload_file(pcloud, fpath, fname, folder)
            .await?
        {
            let reader = std::fs::File::open(fpath).unwrap();
            FileUploadCommand::new(fname, folder.folder_id, reader)
                .no_partial(!self.allow_partial_upload)
                .execute(pcloud)
                .await?;
        }
        if self.remove_after_upload {
            if let Err(error) = std::fs::remove_file(fpath) {
                tracing::error!("unable to delete local file {:?}: {:?}", fpath, error);
            }
        }
        Ok(())
    }

    #[async_recursion(?Send)]
    async fn handle_folder(
        &self,
        pcloud: &HttpClient,
        local_path: &Path,
        remote_folder: &Folder,
    ) -> Result<(), Error> {
        for (fpath, fname) in std::fs::read_dir(local_path)
            .unwrap()
            .filter_map(|entry| entry.ok())
            .map(|entry| entry.path())
            .filter_map(|entry| {
                entry
                    .file_name()
                    .and_then(|fname| fname.to_str())
                    .map(String::from)
                    .map(|fname| (entry, fname))
            })
        {
            if fpath.is_dir() {
                let folder: Folder =
                    FolderCreateCommand::new(fname.to_string(), remote_folder.folder_id)
                        .ignore_exists(true)
                        .execute(pcloud)
                        .await?;
                self.handle_folder(pcloud, &fpath, &folder).await?;
                if self.remove_after_upload {
                    if let Err(error) = std::fs::remove_dir_all(&fpath) {
                        tracing::error!(
                            "unable to delete local directory {:?}: {:?}",
                            fpath,
                            error
                        );
                    }
                }
            } else if fpath.is_file() {
                self.handle_file(pcloud, &fpath, fname.as_str(), &remote_folder)
                    .await?;
            }
        }
        Ok(())
    }

    #[tracing::instrument(skip_all, level = "info")]
    pub async fn execute(&self, pcloud: HttpClient, folder_id: u64) {
        let remote_folder: Folder = FolderListCommand::new(folder_id.into())
            .execute(&pcloud)
            .await
            .expect("couldn't fetch folder");
        self.handle_folder(&pcloud, &self.path, &remote_folder)
            .await
            .expect("couldn't upload folder");
    }
}

#[cfg(all(test, feature = "protected"))]
mod tests {
    use super::Command;
    use crate::tests::*;
    use std::path::{Path, PathBuf};

    #[tokio::test]
    async fn simple() {
        fn build_cmd(root: &Path) -> Command {
            Command {
                remove_after_upload: false,
                allow_partial_upload: false,
                retries: 0,
                path: PathBuf::from(root),
            }
        }
        // prepare basic folder
        let root = create_root();
        let _root_file = create_local_file(root.path(), "foo.txt");
        let first = create_local_dir(root.path(), "first");
        let _first_file = create_local_file(&first, "foo.txt");
        let second = create_local_dir(&first, "second");
        let _second_file = create_local_file(&second, "foo.txt");
        //
        let client = create_client();
        let remote_root = create_remote_dir(&client, 0).await.unwrap();
        //
        build_cmd(root.path())
            .execute(client.clone(), remote_root.folder_id)
            .await;
        //
        let remote_content = scan_remote_dir(&client, remote_root.folder_id)
            .await
            .unwrap();
        assert_eq!(remote_content.len(), 3);
        assert!(remote_content.contains("/foo.txt"));
        assert!(remote_content.contains("/first/foo.txt"));
        assert!(remote_content.contains("/first/second/foo.txt"));
        // add more files locally
        let _third_file = create_local_file(&second, "bar.txt");
        //
        build_cmd(root.path())
            .execute(client.clone(), remote_root.folder_id)
            .await;
        //
        let remote_content = scan_remote_dir(&client, remote_root.folder_id)
            .await
            .unwrap();
        assert_eq!(remote_content.len(), 4);
        assert!(remote_content.contains("/foo.txt"));
        assert!(remote_content.contains("/first/foo.txt"));
        assert!(remote_content.contains("/first/second/foo.txt"));
        assert!(remote_content.contains("/first/second/bar.txt"));
        //
        delete_remote_dir(&client, remote_root.folder_id)
            .await
            .unwrap();
    }

    #[tokio::test]
    async fn removes_after() {
        fn build_cmd(root: &Path) -> Command {
            Command {
                remove_after_upload: true,
                allow_partial_upload: false,
                retries: 0,
                path: PathBuf::from(root),
            }
        }
        // prepare basic folder
        let root = create_root();
        let root_file = create_local_file(root.path(), "foo.txt");
        let first = create_local_dir(root.path(), "first");
        let first_file = create_local_file(&first, "foo.txt");
        let second = create_local_dir(&first, "second");
        let second_file = create_local_file(&second, "foo.txt");
        //
        let client = create_client();
        let remote_root = create_remote_dir(&client, 0).await.unwrap();
        //
        build_cmd(root.path())
            .execute(client.clone(), remote_root.folder_id)
            .await;
        //
        let remote_content = scan_remote_dir(&client, remote_root.folder_id)
            .await
            .unwrap();
        assert_eq!(remote_content.len(), 3);
        assert!(remote_content.contains("/foo.txt"));
        assert!(remote_content.contains("/first/foo.txt"));
        assert!(remote_content.contains("/first/second/foo.txt"));
        //
        assert!(!root_file.exists());
        assert!(!first_file.exists());
        assert!(!second_file.exists());
        //
        delete_remote_dir(&client, remote_root.folder_id)
            .await
            .unwrap();
    }
}
