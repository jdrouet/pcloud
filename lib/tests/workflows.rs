use pcloud::credentials::Credentials;
use pcloud::folder::ROOT;
use pcloud::PCloudApi;
use rand::distributions::Alphanumeric;
use rand::Rng;
use std::io::Cursor;

fn init() {
    let _ = env_logger::builder().is_test(true).try_init();
}

fn create_folder() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
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
    init();
    let folder_name = create_folder();
    let renamed_name = create_folder();
    let child_name = create_folder();
    let client = PCloudApi::new_eu(Credentials::from_env());
    // create folder
    let folder = client.create_folder(&folder_name, ROOT).await.unwrap();
    // rename folder
    let renamed = client
        .rename_folder(folder.folder_id, &renamed_name)
        .await
        .unwrap();
    assert_eq!(folder.folder_id, renamed.folder_id);
    assert_eq!(renamed.name, renamed_name);
    // delete folder
    let deleted = client.delete_folder(folder.folder_id).await.unwrap();
    assert_eq!(deleted.folder_id, folder.folder_id);
    // create folder
    let folder = client.create_folder(&folder_name, ROOT).await.unwrap();
    // create file in folder
    let filename = create_filename("bin");
    let mut filecontent = create_file(1024 * 1024 * 10); // 10Mo
    let cursor = Cursor::new(&mut filecontent);
    let file = client
        .upload_file(cursor, &filename, folder.folder_id)
        .await
        .unwrap();
    // get file info
    let file_info = client.get_info_file(file.file_id).await.unwrap();
    assert_eq!(file_info.metadata.file_id(), Some(file.file_id));
    // get file link
    let _file_link = client.get_link_file(file.file_id).await.unwrap();
    // download file
    let mut buffer: Vec<u8> = Vec::with_capacity(1024 * 1024 * 10);
    let cursor = Cursor::new(&mut buffer);
    let _size = client.download_file(file.file_id, cursor).await.unwrap();
    assert_eq!(buffer, filecontent);
    // rename file
    let renamed_file = client
        .rename_file(file.file_id, "hello.world")
        .await
        .unwrap();
    assert_eq!(renamed_file.name, "hello.world");
    // create other folder
    let child = client
        .create_folder(&child_name, folder.folder_id)
        .await
        .unwrap();
    assert_eq!(Some(folder.folder_id), child.parent_folder_id);
    // create in root folder and move
    let next = client.create_folder(&renamed_name, ROOT).await.unwrap();
    assert_eq!(next.parent_folder_id, Some(ROOT));
    let moved = client
        .move_folder(next.folder_id, folder.folder_id)
        .await
        .unwrap();
    assert_eq!(moved.parent_folder_id, Some(folder.folder_id));
    // delete folder
    let result = client
        .delete_folder_recursive(folder.folder_id)
        .await
        .unwrap();
    assert_eq!(result.deleted_files, 1);
    assert_eq!(result.deleted_folders, 3);
}
