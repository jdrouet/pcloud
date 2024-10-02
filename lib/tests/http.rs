#![cfg(all(feature = "protected", feature = "client-http"))]

use pcloud::client::HttpClientBuilder;
use pcloud::file::checksum::FileCheckSumCommand;
use pcloud::file::download::FileDownloadCommand;
use pcloud::file::rename::FileRenameCommand;
use pcloud::file::upload::FileUploadCommand;
use pcloud::folder::create::FolderCreateCommand;
use pcloud::folder::delete::FolderDeleteCommand;
use pcloud::folder::rename::FolderMoveCommand;
use pcloud::folder::rename::FolderRenameCommand;
use pcloud::folder::ROOT;
use pcloud::prelude::HttpCommand;
use pcloud::streaming::get_file_link::GetFileLinkCommand;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::io::Cursor;

mod common;

fn create_folder() -> String {
    common::random_name()
}

fn create_file(size: usize) -> Vec<u8> {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(size)
        .collect()
}

fn create_filename(ext: &str) -> String {
    format!("{}.{}", create_folder(), ext)
}

#[tokio::test]
async fn complete() {
    common::init();

    let folder_name = create_folder();
    let renamed_name = create_folder();
    let child_name = create_folder();
    let client = HttpClientBuilder::from_env().build().unwrap();
    // create folder
    let folder = FolderCreateCommand::new(folder_name.clone(), ROOT)
        .execute(&client)
        .await
        .unwrap();
    // rename folder
    let renamed = FolderRenameCommand::new(folder.folder_id, renamed_name.clone())
        .execute(&client)
        .await
        .unwrap();
    assert_eq!(folder.folder_id, renamed.folder_id);
    assert_eq!(renamed.base.name, renamed_name);
    // delete folder
    let result = FolderDeleteCommand::new(folder.folder_id.into())
        .execute(&client)
        .await
        .unwrap();
    assert_eq!(result.deleted_folders, 1);
    // let deleted = client.delete_folder(folder.folder_id).await.unwrap();
    // assert_eq!(deleted.folder_id, folder.folder_id);
    // create folder
    let folder = FolderCreateCommand::new(folder_name.clone(), ROOT)
        .execute(&client)
        .await
        .unwrap();
    // create file in folder
    let filename = create_filename("bin");
    let mut filecontent = create_file(1024 * 1024 * 10); // 10Mo
    let cursor = Cursor::new(&mut filecontent);
    let file = FileUploadCommand::new(filename.as_str(), folder.folder_id, cursor)
        .execute(&client)
        .await
        .unwrap();
    // get file info
    let file_info = FileCheckSumCommand::new(file.file_id.into())
        .execute(&client)
        .await
        .unwrap();
    assert_eq!(file_info.metadata.file_id, file.file_id);
    // get file link
    let _file_link = GetFileLinkCommand::new(file.file_id.into())
        .execute(&client)
        .await
        .unwrap();
    // download file
    let mut buffer: Vec<u8> = Vec::with_capacity(1024 * 1024 * 10);
    let cursor = Cursor::new(&mut buffer);
    let _size = FileDownloadCommand::new(file.file_id.into(), cursor)
        .execute(&client)
        .await
        .unwrap();
    assert_eq!(buffer, filecontent);
    // rename file
    let renamed_file = FileRenameCommand::new(file.file_id.into(), "hello.world".into())
        .execute(&client)
        .await
        .unwrap();
    assert_eq!(renamed_file.base.name, "hello.world");
    // create other folder
    let child = FolderCreateCommand::new(child_name.clone(), folder.folder_id)
        .execute(&client)
        .await
        .unwrap();
    assert_eq!(Some(folder.folder_id), child.base.parent_folder_id);
    // create in root folder and move
    let next = FolderCreateCommand::new(renamed_name.clone(), ROOT)
        .execute(&client)
        .await
        .unwrap();
    assert_eq!(next.base.parent_folder_id, Some(ROOT));
    let moved = FolderMoveCommand::new(next.folder_id, folder.folder_id)
        .execute(&client)
        .await
        .unwrap();
    assert_eq!(moved.base.parent_folder_id, Some(folder.folder_id));
    // delete folder
    let result = FolderDeleteCommand::new(folder.folder_id.into())
        .recursive(true)
        .execute(&client)
        .await
        .unwrap();
    assert_eq!(result.deleted_files, 1);
    assert_eq!(result.deleted_folders, 3);
}
