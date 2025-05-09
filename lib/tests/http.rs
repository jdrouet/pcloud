#![cfg(feature = "protected")]

use std::panic;

use pcloud::builder::ClientBuilder;
use pcloud::file::upload::MultiFileUpload;
use pcloud::folder::ROOT;
use pcloud::Credentials;
use rand::distr::Alphanumeric;
use rand::Rng;

mod common;

fn create_folder() -> String {
    common::random_name()
}

fn create_file(size: usize) -> Vec<u8> {
    rand::rng().sample_iter(&Alphanumeric).take(size).collect()
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

    let credentials = Credentials::from_env();
    let Credentials::UsernamePassword { username, password } = credentials else {
        panic!("expect username and password");
    };

    let client = pcloud::Client::default();
    // creates a digest
    let digest = client.get_digest().await.unwrap();

    let credentials = Credentials::username_password_digest(username, digest.value, password);
    let client = client.with_credentials(credentials);

    // fetches the user informations
    let _info = client.user_info().await.unwrap();

    let token = client.get_token().await.unwrap();
    let client = client.with_credentials(Credentials::authorization(token));

    // fetches the user informations
    let _info = client.user_info().await.unwrap();
    // create folder
    let folder = client.create_folder(ROOT, &folder_name).await.unwrap();
    // rename folder
    let renamed = client
        .rename_folder(folder.folder_id, &renamed_name)
        .await
        .unwrap();
    assert_eq!(folder.folder_id, renamed.folder_id);
    assert_eq!(renamed.base.name, renamed_name);
    // delete folder
    client.delete_folder(folder.folder_id).await.unwrap();
    // create folder
    let folder = client.create_folder(ROOT, &folder_name).await.unwrap();
    // create file in folder
    let filename = create_filename("bin");
    let filecontent = create_file(1024 * 1024 * 10); // 10Mo
    let files = MultiFileUpload::default().with_body_entry(filename.as_str(), None, filecontent);
    let mut files = client.upload_files(folder.folder_id, files).await.unwrap();
    let file = files.pop().unwrap();
    // get file info
    let file_info = client.get_file_checksum(file.file_id).await.unwrap();
    assert_eq!(file_info.metadata.file_id, file.file_id);
    // get file link
    let _link = client.get_file_link(file.file_id).await.unwrap();
    // download file
    // let mut buffer: Vec<u8> = Vec::with_capacity(1024 * 1024 * 10);
    // let cursor = Cursor::new(&mut buffer);
    // let _size = FileDownloadCommand::new(file.file_id.into(), cursor)
    //     .execute(&client)
    //     .await
    //     .unwrap();
    // assert_eq!(buffer, filecontent);
    // rename file
    let renamed_file = client
        .rename_file(file.file_id, "hello.world")
        .await
        .unwrap();
    assert_eq!(renamed_file.base.name, "hello.world");
    // create other folder
    let child = client
        .create_folder(folder.folder_id, &child_name)
        .await
        .unwrap();
    assert_eq!(Some(folder.folder_id), child.base.parent_folder_id);
    // create in root folder and move
    let next = client.create_folder(ROOT, &renamed_name).await.unwrap();
    assert_eq!(next.base.parent_folder_id, Some(ROOT));
    let moved = client
        .move_folder(next.folder_id, folder.folder_id)
        .await
        .unwrap();
    assert_eq!(moved.base.parent_folder_id, Some(folder.folder_id));
    // delete folder
    let result = client
        .delete_folder_recursive(folder.folder_id)
        .await
        .unwrap();
    assert_eq!(result.deleted_files, 1);
    assert_eq!(result.deleted_folders, 3);
}
