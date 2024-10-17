use std::fmt::Display;

use human_bytes::human_bytes;
use pcloud::entry::{Entry, File, Folder};

use crate::FolderCloudPath;

mod macros;

const DATE_FORMAT: &str = "%Y-%b-%d %T";

fn render_file(
    f: &mut std::fmt::Formatter<'_>,
    ctx: &PageContext,
    file: &File,
) -> std::fmt::Result {
    crate::node!(f, "tr", {
        crate::node!(f, "td", "class=\"n\"", {
            write!(
                f,
                r#"<a href="{}{}">{}</a>"#,
                ctx.prefix,
                ctx.path.with_encoded_file(file.base.name.as_str()),
                file.base.name.as_str(),
            )?;
        });
        crate::node!(f, "td", "class=\"m\"", {
            file.base.modified.format(DATE_FORMAT).fmt(f)?;
        });
        crate::node!(f, "td", "class=\"s\"", {
            let size = file.size.unwrap_or(0);
            human_bytes(size as f64).fmt(f)?;
        });
        crate::node!(f, "td", "class=\"t\"", {
            if let Some(ref ctype) = file.content_type {
                f.write_str(ctype)?;
            }
        });
    });
    Ok(())
}

fn render_folder(
    f: &mut std::fmt::Formatter<'_>,
    ctx: &PageContext,
    folder: &Folder,
) -> std::fmt::Result {
    crate::node!(f, "tr", {
        crate::node!(f, "td", "class=\"n\"", {
            write!(
                f,
                r#"<a href="{}{}">{}</a>"#,
                ctx.prefix,
                ctx.path.with_encoded_folder(folder.base.name.as_str()),
                folder.base.name.as_str(),
            )?;
        });
        crate::node!(f, "td", "class=\"m\"", {
            folder.base.modified.format(DATE_FORMAT).fmt(f)?;
        });
        crate::node!(f, "td", "class=\"s\"", {
            f.write_str("- &nbsp;")?;
        });
        crate::node!(f, "td", "class=\"t\"", {
            f.write_str("Directory")?;
        });
    });
    Ok(())
}

fn render_entry(
    f: &mut std::fmt::Formatter<'_>,
    ctx: &PageContext,
    entry: &Entry,
) -> std::fmt::Result {
    match entry {
        Entry::File(file) => render_file(f, ctx, file),
        Entry::Folder(folder) => render_folder(f, ctx, folder),
    }
}

struct PageContext<'a> {
    prefix: &'static str,
    path: &'a FolderCloudPath,
}

pub(crate) struct IndexPage<'a> {
    ctx: PageContext<'a>,
    folder: &'a Folder,
}

impl<'a> IndexPage<'a> {
    pub fn from_folder_list(
        prefix: &'static str,
        path: &'a FolderCloudPath,
        folder: &'a Folder,
    ) -> Self {
        Self {
            ctx: PageContext { prefix, path },
            folder,
        }
    }
}

impl<'a> IndexPage<'a> {
    #[inline(always)]
    fn render_head(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(
            f,
            "<title>Index of {}{}</title>",
            self.ctx.prefix,
            self.ctx.path.raw()
        )?;
        f.write_str("<style>")?;
        f.write_str(include_str!("style.css"))?;
        f.write_str("</style>")
    }

    #[inline(always)]
    fn render_body(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        crate::node!(f, "header", {
            write!(
                f,
                "<h2>Index of {}{}</h2>",
                self.ctx.prefix,
                self.ctx.path.raw()
            )?;
        });
        crate::node!(f, "main", {
            crate::node!(
                f,
                "table",
                "summary=\"Directory Listing\" cellpadding=\"0\" cellspacing=\"0\"",
                {
                    crate::node!(f, "thead", {
                        crate::node!(f, "tr", {
                            f.write_str("<th class=\"n\">Name</th>")?;
                            f.write_str("<th class=\"m\">Last Modified</th>")?;
                            f.write_str("<th class=\"s\">Size</th>")?;
                            f.write_str("<th class=\"t\">Type</th>")?;
                        });
                    });
                    crate::node!(f, "tbody", {
                        self.render_rows(f)?;
                    });
                }
            );
            f.write_str("</table>")?;
        });
        f.write_str("<div class=\"foot\"></div>")?;
        Ok(())
    }

    #[inline(always)]
    fn render_rows(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        let mut rows = self.folder.contents.iter().flatten().collect::<Vec<_>>();
        rows.sort();
        for entry in rows {
            render_entry(f, &self.ctx, entry)?;
        }
        Ok(())
    }
}

impl<'a> std::fmt::Display for IndexPage<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.write_str("<!DOCTYPE html>")?;
        crate::node!(f, "html", {
            crate::node!(f, "head", {
                self.render_head(f)?;
            });
            crate::node!(f, "body", {
                self.render_body(f)?;
            });
        });
        Ok(())
    }
}
