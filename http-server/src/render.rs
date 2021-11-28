use human_bytes::human_bytes;
use pcloud::entry::{Entry, File, Folder};

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

pub fn format_page(path: &str, folder: &Folder) -> String {
    let mut children = folder.contents.clone().unwrap_or_default();
    children.sort();
    let children = children
        .iter()
        .map(format_entry_row)
        .collect::<Vec<_>>()
        .join("");
    format!(
        r#"<!DOCTYPE html>
<html>
<head>
<meta charset="utf-8">
<title>Index of {path}</title>
<style>table {{ width: 100%; }}</style>
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
