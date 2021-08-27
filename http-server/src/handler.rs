use actix_web::{web, HttpResponse};
use human_bytes::human_bytes;
use pcloud::entry::{Entry, File, Folder};
use pcloud::folder::list::Params as ListFolderParams;
use pcloud::http::PCloudHttpApi;

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

const DATE_FORMAT: &str = "%Y-%b-%d %T";

fn format_file_row(file: &File) -> String {
    format!(
        r#"<tr><td class="n"><a href="{name}">{name}</a></td><td class="m">{modified}</td><td class="s">{size}</td><td class="t">{content_type}</td></tr>"#,
        name = file.base.name,
        modified = file.base.modified.format(DATE_FORMAT),
        size = human_bytes(file.size.unwrap_or(0) as f64),
        content_type = file.content_type.clone().unwrap_or_default()
    )
}

fn format_folder_row(folder: &Folder) -> String {
    format!(
        r#"<tr><td class="n"><a href="{name}/">{name}</a></td><td class="m">{modified}</td><td class="s">- &nbsp;</td><td class="t">Directory</td></tr>"#,
        name = folder.base.name,
        modified = folder.base.modified.format(DATE_FORMAT),
    )
}

fn format_entry_row(entry: &Entry) -> String {
    match entry {
        Entry::File(file) => format_file_row(file),
        Entry::Folder(folder) => format_folder_row(folder),
    }
}

fn format_page(path: &str, folder: &Folder) -> String {
    let children = folder
        .contents
        .as_ref()
        .map(|contents| contents.iter().map(format_entry_row).collect::<Vec<_>>())
        .unwrap_or_default()
        .join("");
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Index of {path}</title>
</head>
<body>
<h2>Index of {path}</h2>
<div class="list">
<table summary="Directory Listing" cellpadding="0" cellspacing="0">
<thead><tr><th class="n">Name</th><th class="m">Last Modified</th><th class="s">Size</th><th class="t">Type</th></tr></thead>
<tbody>
{children}
</tbody>
</table>
</div>
<div class="foot"> </div>
</body>
</html>
"#,
        path = path,
        children = children
    )
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

pub async fn handle(
    client: web::Data<PCloudHttpApi>,
    root: web::Data<RootFolder>,
    path: web::Path<String>,
) -> HttpResponse {
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
        match client.list_folder(&params).await {
            Ok(folder) => HttpResponse::Ok().body(format_page(&original_path, &folder)),
            Err(err) => HttpResponse::BadGateway().body(format!("unable to get folder: {:?}", err)),
        }
    } else {
        match client.get_link_file(target_path).await {
            Ok(url) => HttpResponse::Found()
                .append_header(("Location", url))
                .finish(),
            Err(err) => HttpResponse::BadGateway().body(format!("unable to get file: {:?}", err)),
        }
    }
}
