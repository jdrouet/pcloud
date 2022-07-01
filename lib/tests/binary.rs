#![cfg(all(
    feature = "client-binary",
    feature = "client-http",
    feature = "protected",
))]

use pcloud::binary::BinaryClientBuilder;
use pcloud::file::download::FileDownloadCommand;
use pcloud::fileops::close::FileCloseCommand;
use pcloud::fileops::open::FileOpenCommand;
use pcloud::fileops::pread::FilePReadCommand;
use pcloud::folder::list::FolderListCommand;
use pcloud::http::HttpClientBuilder;

mod common;

async fn http_download_file() -> (u64, Vec<u8>) {
    use pcloud::prelude::HttpCommand;

    let client = HttpClientBuilder::from_env().build().unwrap();
    let folder = FolderListCommand::new(0.into())
        .execute(&client)
        .await
        .unwrap();
    let file = folder.find_file("landscape.jpeg").unwrap();
    let mut data = Vec::new();
    FileDownloadCommand::new(file.file_id.into(), &mut data)
        .execute(&client)
        .await
        .unwrap();
    (file.file_id, data)
}

#[tokio::test]
async fn downloading_with_read() {
    use pcloud::prelude::BinaryCommand;

    common::init();

    let (file_id, expected) = http_download_file().await;
    //
    let mut client = BinaryClientBuilder::from_env().build().unwrap();
    //
    let fd = FileOpenCommand::new(0)
        .identifier(file_id.into())
        .execute(&mut client)
        .unwrap();
    // size requested by fuse
    let loop_size = 64 * 1024;
    let mut result: Vec<u8> = Vec::new();
    let mut offset = 0;
    while result.len() < expected.len() {
        let data = FilePReadCommand::new(fd, loop_size, offset)
            .execute(&mut client)
            .unwrap();
        offset += data.len();
        result.extend(&data);
    }
    FileCloseCommand::new(fd).execute(&mut client).unwrap();
    assert_eq!(result, expected);
}
