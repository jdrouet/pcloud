use crate::{RootPrefix, Storage};
use axum::extract::Query;
use axum::response::{Html, IntoResponse};
use axum::{extract::Path, Extension};
use pcloud::entry::File;
use pcloud::file::FileIdentifier;
use pcloud::folder::list::FolderListCommand;
use pcloud::prelude::HttpCommand;
use std::path::PathBuf;
use std::str::FromStr;
use std::string::FromUtf8Error;
use std::sync::Arc;

const PREFIX: &str = "/by-path";

#[derive(serde::Serialize)]
struct ErrorReponse {
    message: String,
    details: Option<String>,
}

#[derive(Debug, thiserror::Error)]
pub enum Error {
    #[error("unable to path requested file path")]
    InvalidPath(#[source] FromUtf8Error),
    #[error("unable to list files in folder")]
    UnableListFolder(#[source] pcloud::error::Error),
    #[error("unable to get file")]
    UnableGetFile(#[source] pcloud::error::Error),
    #[error("unable to get file link")]
    UnableGetLink,
}

impl Error {
    fn status_code(&self) -> axum::http::StatusCode {
        match self {
            // Bad request
            Self::UnableGetFile(pcloud::error::Error::Protocol(2010, _))
            | Self::UnableListFolder(pcloud::error::Error::Protocol(2010, _))
            | Self::InvalidPath(_) => axum::http::StatusCode::BAD_REQUEST,
            // Unauthorized
            Self::UnableGetFile(pcloud::error::Error::Protocol(1000, _))
            | Self::UnableListFolder(pcloud::error::Error::Protocol(1000, _))
            | Self::UnableGetFile(pcloud::error::Error::Protocol(2000, _))
            | Self::UnableListFolder(pcloud::error::Error::Protocol(2000, _)) => {
                axum::http::StatusCode::UNAUTHORIZED
            }
            // Forbidden
            Self::UnableGetFile(pcloud::error::Error::Protocol(2003, _))
            | Self::UnableListFolder(pcloud::error::Error::Protocol(2003, _)) => {
                axum::http::StatusCode::FORBIDDEN
            }
            // Not found
            Self::UnableGetFile(pcloud::error::Error::Protocol(2009, _))
            | Self::UnableListFolder(pcloud::error::Error::Protocol(2005, _)) => {
                axum::http::StatusCode::NOT_FOUND
            }
            _ => axum::http::StatusCode::INTERNAL_SERVER_ERROR,
        }
    }

    fn details(&self) -> Option<String> {
        match self {
            Self::UnableGetFile(inner) | Self::UnableListFolder(inner) => Some(inner.to_string()),
            _ => None,
        }
    }

    fn response(&self) -> ErrorReponse {
        ErrorReponse {
            message: self.to_string(),
            details: self.details(),
        }
    }
}

impl axum::response::IntoResponse for Error {
    fn into_response(self) -> axum::response::Response {
        let status = self.status_code();
        let body = self.response();

        (status, axum::Json(body)).into_response()
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
        let list = if params.stream {
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

        let list = list.map_err(Error::UnableGetFile)?;
        let link = list.first_link().ok_or(Error::UnableGetLink)?;

        Ok(Success::File(link.to_string()))
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
