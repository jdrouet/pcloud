use human_bytes::human_bytes;
use pcloud::entry::{Entry, File, Folder};

use crate::FolderCloudPath;

const DATE_FORMAT: &str = "%Y-%b-%d %T";

pub(crate) enum EntrySize {
    File(usize),
    Folder,
}

impl std::fmt::Display for EntrySize {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(size) => write!(f, "{}", human_bytes(*size as f64)),
            Self::Folder => f.write_str("- &nbsp;"),
        }
    }
}

pub(crate) enum ContentType<'a> {
    File(Option<&'a str>),
    Directory,
}

impl<'a> std::fmt::Display for ContentType<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            Self::File(Some(inner)) => f.write_str(inner),
            Self::File(None) => Ok(()),
            Self::Directory => f.write_str("Directory"),
        }
    }
}

pub(crate) struct EntryRow<'a> {
    href: String,
    name: &'a str,
    modified: String,
    size: EntrySize,
    content_type: ContentType<'a>,
}

impl<'a> std::fmt::Display for EntryRow<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let Self {
            href,
            name,
            modified,
            size,
            content_type,
        } = self;

        write!(
            f,
            r#"<tr><td class="n"><a href="{href}">{name}</a></td><td class="m">{modified}</td><td class="s">{size}</td><td class="t">{content_type}</td></tr>"#,
        )
    }
}

impl<'a> EntryRow<'a> {
    pub fn from_file(prefix: &'static str, path: &'a FolderCloudPath, entry: &'a File) -> Self {
        Self {
            href: format!("{prefix}{}", path.with_file(entry.base.name.as_str())),
            name: entry.base.name.as_str(),
            modified: entry.base.modified.format(DATE_FORMAT).to_string(),
            size: EntrySize::File(entry.size.unwrap_or(0)),
            content_type: ContentType::File(entry.content_type.as_deref()),
        }
    }

    pub fn from_folder(prefix: &'static str, path: &'a FolderCloudPath, entry: &'a Folder) -> Self {
        Self {
            href: format!("{prefix}{}", path.with_folder(entry.base.name.as_str())),
            name: entry.base.name.as_str(),
            modified: entry.base.modified.format(DATE_FORMAT).to_string(),
            size: EntrySize::Folder,
            content_type: ContentType::Directory,
        }
    }

    pub fn from_entry(prefix: &'static str, path: &'a FolderCloudPath, entry: &'a Entry) -> Self {
        match entry {
            Entry::File(inner) => Self::from_file(prefix, path, inner),
            Entry::Folder(inner) => Self::from_folder(prefix, path, inner),
        }
    }
}

pub(crate) struct IndexPage<'a> {
    prefix: &'static str,
    path: &'a FolderCloudPath,
    rows: Vec<EntryRow<'a>>,
}

impl<'a> IndexPage<'a> {
    pub fn from_folder_list(
        prefix: &'static str,
        path: &'a FolderCloudPath,
        folder: &'a Folder,
    ) -> Self {
        if let Some(ref inner) = folder.contents {
            let rows = inner
                .iter()
                .map(|row| EntryRow::from_entry(prefix, path, row))
                .collect();
            Self { prefix, path, rows }
        } else {
            Self {
                prefix,
                path,
                rows: Vec::new(),
            }
        }
    }
}

impl<'a> std::fmt::Display for IndexPage<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<!DOCTYPE html>")?;
        f.write_str("<html>")?;
        f.write_str("<head>")?;
        f.write_str(r#"<meta charset="utf-8" />"#)?;
        write!(f, "<title>Index of {}{}</title>", self.prefix, self.path)?;
        f.write_str("<style>table { width: 100%; }</style>")?;
        f.write_str("</head>")?;
        f.write_str("<body>")?;
        write!(f, "<h2>Index of {}{}</h2>", self.prefix, self.path)?;
        f.write_str("<div class=\"list\">")?;
        f.write_str("<table summary=\"Directory Listing\" cellpadding=\"0\" cellspacing=\"0\">")?;
        f.write_str("<thead>")?;
        f.write_str("<tr>")?;
        f.write_str("<th class=\"n\">Name</th>")?;
        f.write_str("<th class=\"m\">Last Modified</th>")?;
        f.write_str("<th class=\"s\">Size</th>")?;
        f.write_str("<th class=\"t\">Type</th>")?;
        f.write_str("</tr>")?;
        f.write_str("</thead>")?;
        f.write_str("<tbody>")?;
        for row in self.rows.iter() {
            row.fmt(f)?;
        }
        f.write_str("</tbody>")?;
        f.write_str("</table>")?;
        f.write_str("</div>")?;
        f.write_str("<div class=\"foot\"></div>")?;
        f.write_str("</body>")?;
        f.write_str("</html>")?;
        Ok(())
    }
}
