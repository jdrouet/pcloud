use crate::{RootPrefix, Storage};
use axum::extract::{Path, Query};
use axum::response::{Html, IntoResponse};
use axum::Extension;
use pcloud::file::File;
use std::str::FromStr;
use std::string::FromUtf8Error;

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
    UnableListFolder(#[source] pcloud::Error),
    #[error("unable to get file")]
    UnableGetFile(#[source] pcloud::Error),
    #[error("unable to get file link")]
    UnableGetLink,
}

impl Error {
    fn status_code(&self) -> axum::http::StatusCode {
        match self {
            // Bad request
            Self::UnableGetFile(pcloud::Error::Protocol(2010, _))
            | Self::UnableListFolder(pcloud::Error::Protocol(2010, _))
            | Self::InvalidPath(_) => axum::http::StatusCode::BAD_REQUEST,
            // Unauthorized
            Self::UnableGetFile(pcloud::Error::Protocol(1000, _))
            | Self::UnableListFolder(pcloud::Error::Protocol(1000, _))
            | Self::UnableGetFile(pcloud::Error::Protocol(2000, _))
            | Self::UnableListFolder(pcloud::Error::Protocol(2000, _)) => {
                axum::http::StatusCode::UNAUTHORIZED
            }
            // Forbidden
            Self::UnableGetFile(pcloud::Error::Protocol(2003, _))
            | Self::UnableListFolder(pcloud::Error::Protocol(2003, _)) => {
                axum::http::StatusCode::FORBIDDEN
            }
            // Not found
            Self::UnableGetFile(pcloud::Error::Protocol(2009, _))
            | Self::UnableListFolder(pcloud::Error::Protocol(2005, _)) => {
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
        let folder_content = engine
            .as_ref()
            .list_folder(remote_path.into_inner().raw().to_string())
            .await
            .map_err(Error::UnableListFolder)?;
        Ok(Success::Directory(
            crate::render::IndexPage::from_folder_list(PREFIX, &local_path, &folder_content)
                .to_string(),
        ))
    } else {
        let local_path = crate::CloudPath::from_str(path).map_err(Error::InvalidPath)?;
        let remote_path = root_prefix.root_path().join_file(local_path);
        let identifier = remote_path.raw().to_string();
        let list = if params.stream {
            let file = engine
                .as_ref()
                .get_file_checksum(&identifier)
                .await
                .map_err(Error::UnableGetFile)?;

            if is_video(&file.metadata) {
                engine.as_ref().get_video_link(identifier).await
            } else if is_audio(&file.metadata) {
                engine.as_ref().get_audio_link(identifier).await
            } else {
                engine.as_ref().get_file_link(identifier).await
            }
        } else {
            engine.as_ref().get_file_link(identifier).await
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
