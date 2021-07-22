use pcloud::credentials::Credentials;
use pcloud::folder::{ListFolderParams, ROOT};
use pcloud::PCloudApi;
use rand::distributions::Alphanumeric;
use rand::Rng;

fn create_folder() -> String {
    rand::thread_rng()
        .sample_iter(&Alphanumeric)
        .take(16)
        .map(char::from)
        .collect()
}

fn create_filename(ext: &str) -> String {
    format!("{}.{}", create_folder(), ext)
}

#[tokio::test]
async fn folder() {
    let folder_name = create_folder();
    let renamed_name = create_folder();
    let child_name = create_folder();
    let client = PCloudApi::new_eu(Credentials::from_env());
    // create folder
    let folder = client.create_folder(&folder_name, ROOT).await.unwrap();
    assert!(folder.is_folder);
    // rename folder
    let renamed = client
        .rename_folder(folder.folder_id.unwrap(), &renamed_name)
        .await
        .unwrap();
    assert_eq!(folder.folder_id, renamed.folder_id);
    assert_eq!(renamed.name, renamed_name);
    // delete folder
    let deleted = client
        .delete_folder(folder.folder_id.unwrap())
        .await
        .unwrap();
    assert_eq!(deleted.folder_id, folder.folder_id);
    // create folder
    let folder = client.create_folder(&folder_name, ROOT).await.unwrap();
    assert!(folder.is_folder);
    // create other folder
    let child = client
        .create_folder(&child_name, folder.folder_id.unwrap())
        .await
        .unwrap();
    assert_eq!(folder.folder_id, child.parent_folder_id);
    // create in root folder and move
    let next = client.create_folder(&renamed_name, ROOT).await.unwrap();
    assert_eq!(next.parent_folder_id, Some(ROOT));
    let moved = client
        .move_folder(next.folder_id.unwrap(), folder.folder_id.unwrap())
        .await
        .unwrap();
    assert_eq!(moved.parent_folder_id, folder.folder_id);
    // delete folder
    let (deleted_files, deleted_folders) = client
        .delete_folder_recursive(folder.folder_id.unwrap())
        .await
        .unwrap();
    assert_eq!(deleted_files, 0);
    assert_eq!(deleted_folders, 3);
}
