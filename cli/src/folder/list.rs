use clap::Clap;
use pcloud::entry::Entry;
use pcloud::http::PCloudApi;

#[derive(Clap)]
pub struct Command;

impl Command {
    fn print(&self, mut result: Vec<Entry>) {
        result.sort();
        println!(
            "{:<12} {:<6} {:<32} {:<20}",
            "ID", "Type", "Name", "Updated at"
        );
        for entry in result.iter() {
            let id = match entry {
                Entry::File(file) => file.file_id,
                Entry::Folder(folder) => folder.folder_id,
            };
            let type_ = match entry {
                Entry::File(_) => "file",
                Entry::Folder(_) => "folder",
            };
            println!(
                "{:<12} {:<6} {:<32} {:<20}",
                id,
                type_,
                entry.base().name,
                entry.base().modified,
            );
        }
    }

    pub async fn execute(&self, pcloud: PCloudApi, folder_id: usize) {
        match pcloud.list_folder(folder_id).await {
            Ok(res) => {
                self.print(res.contents.unwrap_or_default());
                std::process::exit(exitcode::OK);
            }
            Err(err) => {
                log::error!("unable to delete folder: {:?}", err);
                std::process::exit(exitcode::DATAERR);
            }
        }
    }
}
