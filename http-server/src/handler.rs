use crate::render;
use actix_web::error::ResponseError;
use actix_web::{web, HttpRequest, HttpResponse, HttpResponseBuilder};
use pcloud::folder::list::Params as ListFolderParams;
use pcloud::http::HttpClient;
use std::fmt;

#[derive(Debug)]
pub enum Error {
    UnableListFolder(pcloud::error::Error),
    UnableGetFile(pcloud::error::Error),
    OpenStream(reqwest::Error),
}

impl fmt::Display for Error {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        match self {
            Self::UnableListFolder(err) => {
                write!(f, "unable to list folder: {:?}", err)
            }
            Self::UnableGetFile(err) => {
                write!(f, "unable to get file: {:?}", err)
            }
            Self::OpenStream(err) => {
                write!(f, "unable to open remote stream: {:?}", err)
            }
        }
    }
}

impl ResponseError for Error {
    fn error_response(&self) -> HttpResponse {
        match self {
            Self::UnableListFolder(err) => {
                HttpResponse::BadGateway().body(format!("unable to get folder: {:?}", err))
            }
            Self::UnableGetFile(err) => {
                HttpResponse::BadGateway().body(format!("unable to get file: {:?}", err))
            }
            Self::OpenStream(err) => {
                HttpResponse::NotFound().body(format!("unable to open remote stream: {:?}", err))
            }
        }
    }
}

pub struct RootFolder(String);

impl RootFolder {
    pub fn from_env() -> Self {
        Self(std::env::var("ROOT_FOLDER").unwrap_or_else(|_| "/".into()))
    }

    pub fn format(&self, path: &str) -> String {
        let root = self.0.strip_suffix('/').unwrap_or(self.0.as_str());
        let sub = path.strip_prefix('/').unwrap_or(path);
        format!("{}/{}", root, sub)
    }
}

fn format_path(path: &str) -> String {
    if path.is_empty() {
        "/".into()
    } else if !path.starts_with('/') {
        format!("/{}", path)
    } else {
        path.to_string()
    }
}

fn build_stream_request(origin: &HttpRequest, url: &str) -> reqwest::RequestBuilder {
    let client = reqwest::Client::new();
    origin
        .headers()
        .iter()
        .fold(client.get(url), |r, (name, value)| {
            r.header(name.as_str(), value.as_bytes())
        })
}

fn build_stream_response(response: &reqwest::Response) -> HttpResponseBuilder {
    let mut res = HttpResponseBuilder::new(response.status());
    response.headers().iter().for_each(|(name, value)| {
        res.append_header((name, value));
    });
    res
}

pub async fn handle(
    req: HttpRequest,
    client: web::Data<HttpClient>,
    root: web::Data<RootFolder>,
    path: web::Path<String>,
) -> Result<HttpResponse, Error> {
    let original_path = format_path(path.as_str());
    log::info!("handle path={}", original_path);
    let target_path = root.format(original_path.as_str());
    log::debug!("forward to path={}", target_path);

    if let Some(path) = target_path.strip_suffix('/') {
        let params = ListFolderParams::new(if path.is_empty() {
            target_path.to_string()
        } else {
            path.to_string()
        });
        client
            .list_folder(&params)
            .await
            .map(|folder| HttpResponse::Ok().body(render::format_page(&original_path, &folder)))
            .map_err(Error::UnableListFolder)
    } else {
        let result = client.get_link_file(target_path).await;
        let url = result.map_err(Error::UnableGetFile)?;
        let redirect = build_stream_request(&req, &url);
        let redirect = redirect.send().await.map_err(Error::OpenStream)?;
        let mut res = build_stream_response(&redirect);
        let stream = redirect.bytes_stream();
        Ok(res.streaming(stream))
    }
}
