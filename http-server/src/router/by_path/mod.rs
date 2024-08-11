use crate::{RootPrefix, Storage};
use axum::extract::Query;
use axum::response::{Html, IntoResponse};
use axum::{extract::Path, Extension};
use pcloud::entry::File;
use pcloud::file::FileIdentifier;
use pcloud::folder::list::FolderListCommand;
use pcloud::prelude::HttpCommand;
use std::fmt;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::FromUtf8Error;
use std::sync::Arc;

const PREFIX: &str = "/by-path";

#[derive(Debug)]
pub enum Error {
    InvalidPath(FromUtf8Error),
    UnableListFolder(pcloud::error::Error),
    UnableGetFile(pcloud::error::Error),
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        eprintln!("error: {self:?}");
        axum::http::StatusCode::INTERNAL_SERVER_ERROR.into_response()
    }
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::InvalidPath(err) => {
                write!(f, "unable to parse file path: {err:?}")
            }
            Self::UnableListFolder(err) => {
                write!(f, "unable to list folder: {err:?}")
            }
            Self::UnableGetFile(err) => {
                write!(f, "unable to get file: {err:?}")
            }
        }
    }
}

#[derive(Debug, Clone)]
pub struct RootFolder(Arc<PathBuf>);

impl RootFolder {
    pub fn new(value: PathBuf) -> Self {
        Self(Arc::new(value))
    }

    pub fn join(&self, child: &str) -> String {
        self.0.join(child).to_str().unwrap().to_string()
    }

    pub fn as_str(&self) -> String {
        self.0.to_str().unwrap().to_string()
    }
}

pub(crate) enum Success {
    Directory(String),
    File(String),
}

impl IntoResponse for Success {
    fn into_response(self) -> axum::response::Response {
        match self {
            Self::Directory(inner) => Html(inner).into_response(),
            Self::File(inner) => {
                axum::response::Redirect::temporary(inner.as_str()).into_response()
            }
        }
    }
}

fn is_video(file: &File) -> bool {
    match file.content_type {
        Some(ref inner) => inner.starts_with("video/"),
        _ => false,
    }
}

fn is_audio(file: &File) -> bool {
    match file.content_type {
        Some(ref inner) => inner.starts_with("audio/"),
        _ => false,
    }
}

async fn handle(
    engine: Storage,
    root_prefix: RootPrefix,
    path: &str,
    params: QueryParams,
) -> Result<Success, Error> {
    if path.ends_with('/') {
        let local_path = crate::FolderCloudPath::from_str(path).map_err(Error::InvalidPath)?;
        let remote_path = root_prefix.root_path().join_folder(local_path.clone());
        // FolderListCommand shouldn't get '/' at the end of path
        let folder_content = FolderListCommand::new(remote_path.into_inner().to_string().into())
            .execute(engine.as_ref())
            .await
            .map_err(Error::UnableListFolder)?;
        Ok(Success::Directory(
            crate::render::IndexPage::from_folder_list(PREFIX, &local_path, &folder_content)
                .to_string(),
        ))
    } else {
        let local_path = crate::CloudPath::from_str(path).map_err(Error::InvalidPath)?;
        let remote_path = root_prefix.root_path().join_file(local_path);

        let identifier: FileIdentifier = remote_path.to_string().into();
        let link = if params.stream {
            let file = pcloud::file::checksum::FileCheckSumCommand::new(identifier.clone())
                .execute(engine.as_ref())
                .await
                .map_err(Error::UnableGetFile)?;

            if is_video(&file.metadata) {
                pcloud::streaming::get_video_link::GetVideoLinkCommand::new(identifier)
                    .execute(engine.as_ref())
                    .await
            } else if is_audio(&file.metadata) {
                pcloud::streaming::get_audio_link::GetAudioLinkCommand::new(identifier)
                    .execute(engine.as_ref())
                    .await
            } else {
                pcloud::streaming::get_file_link::GetFileLinkCommand::new(identifier)
                    .execute(engine.as_ref())
                    .await
            }
        } else {
            pcloud::streaming::get_file_link::GetFileLinkCommand::new(identifier)
                .execute(engine.as_ref())
                .await
        };

        let link = link.map_err(Error::UnableGetFile)?;

        Ok(Success::File(link))
    }
}

pub(crate) async fn index_handler(
    Extension(engine): Extension<Storage>,
    Extension(root_prefix): Extension<RootPrefix>,
) -> Result<Success, Error> {
    handle(engine, root_prefix, "/", QueryParams::default()).await
}

#[derive(Debug, Default, serde::Deserialize)]
pub struct QueryParams {
    #[serde(default)]
    stream: bool,
}

pub(crate) async fn any_handler(
    Extension(engine): Extension<Storage>,
    Extension(root_prefix): Extension<RootPrefix>,
    Path(path): Path<String>,
    Query(params): Query<QueryParams>,
) -> Result<Success, Error> {
    handle(engine, root_prefix, path.as_str(), params).await
}
