use pcloud::{
    entry::{Entry, File},
    prelude::HttpCommand,
};

#[derive(Default)]
struct ColumnWidths {
    kind: usize,
    name: usize,
    size: usize,
    modified: usize,
}

impl ColumnWidths {
    fn from_iter<'a>(iter: impl Iterator<Item = &'a EntryLine<'a>>) -> Self {
        iter.fold(Self::default(), |mut res, item| {
            res.kind = res.kind.max(item.kind.len());
            res.name = res.name.max(item.name.len());
            if let Some(ref size) = item.size {
                res.size = res.size.max(size.len());
            }
            res.modified = res.modified.max(item.modified.len());
            res
        })
    }
}

struct ListFormatter {
    content_type: bool,
    size_fmt: human_number::Formatter<'static>,
}

impl ListFormatter {
    fn file_kind<'a>(&self, entry: &'a File) -> &'a str {
        if self.content_type {
            entry
                .content_type
                .as_deref()
                .map(|v| v.split_once('/').map_or(v, |(left, _)| left))
                .unwrap_or("file")
        } else {
            "file"
        }
    }

    fn convert_lines<'a>(&self, entries: &'a [Entry]) -> Vec<EntryLine<'a>> {
        entries
            .iter()
            .map(move |entry| match entry {
                Entry::File(inner) => EntryLine {
                    kind: self.file_kind(inner),
                    name: inner.base.name.as_str(),
                    size: inner
                        .size
                        .map(|value| self.size_fmt.format(value as f64).to_string()),
                    modified: inner.base.modified.to_rfc3339(),
                },
                Entry::Folder(inner) => EntryLine {
                    kind: "directory",
                    name: inner.base.name.as_str(),
                    size: None,
                    modified: inner.base.modified.to_rfc3339(),
                },
            })
            .collect::<Vec<_>>()
    }

    fn write(&self, entries: Vec<Entry>) {
        let lines = self.convert_lines(&entries);
        let column_widths = ColumnWidths::from_iter(lines.iter());
        for line in lines {
            let size = line.size.as_deref().unwrap_or("-");
            println!(
                "{:<kw$}  {:nw$}  {:>sw$}  {:mw$}",
                line.kind,
                line.name,
                size,
                line.modified,
                kw = column_widths.kind,
                nw = column_widths.name,
                sw = column_widths.size,
                mw = column_widths.modified,
            );
        }
    }
}

struct EntryLine<'a> {
    kind: &'a str,
    name: &'a str,
    size: Option<String>,
    modified: String,
}

#[derive(clap::Parser)]
pub(crate) struct Command {
    /// Format size to be human readable
    #[clap(long, default_value = "false")]
    human_size: bool,

    /// Display files content type
    #[clap(long, default_value = "false")]
    content_type: bool,

    /// Remote path to list
    #[clap(default_value = "/")]
    path: String,
}

impl Command {
    async fn fetch(
        &self,
        client: &pcloud::client::HttpClient,
    ) -> Result<Vec<Entry>, pcloud::error::Error> {
        // assuming it's doing a `ls` on a folder at first
        let folder_id = pcloud::folder::FolderIdentifier::path(&self.path);
        let folder_res = pcloud::folder::list::FolderListCommand::new(folder_id)
            .execute(client)
            .await;
        match folder_res {
            Ok(folder) => Ok(folder.contents.unwrap_or_default()),
            Err(pcloud::error::Error::Protocol(2005, _)) => {
                // try with a file if a folder is not found
                let file_id = pcloud::file::FileIdentifier::path(&self.path);
                pcloud::file::checksum::FileCheckSumCommand::new(file_id)
                    .execute(client)
                    .await
                    .map(|res| vec![Entry::File(res.metadata)])
            }
            Err(inner) => Err(inner),
        }
    }

    fn formatter(&self) -> ListFormatter {
        ListFormatter {
            content_type: self.content_type,
            size_fmt: if self.human_size {
                human_number::Formatter::binary().with_unit("B")
            } else {
                human_number::Formatter::empty().with_decimals(0)
            },
        }
    }

    pub(crate) async fn execute(self, client: &pcloud::client::HttpClient) -> anyhow::Result<()> {
        let result = self.fetch(client).await?;
        self.formatter().write(result);
        Ok(())
    }
}
