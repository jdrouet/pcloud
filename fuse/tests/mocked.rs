use fuser::MountOption;
use mockito::*;
use std::fs::{File, OpenOptions};
use std::io::Write;

fn init() {
    let _ = tracing_subscriber::fmt()
        .with_env_filter(std::env::var("RUST_LOG").unwrap_or_else(|_| String::from("debug")))
        .try_init()
        .ok();
}

fn build_client() -> pcloud::http::HttpClient {
    let region = pcloud::region::Region::new(mockito::server_url(), "foo".into());
    let creds = pcloud::credentials::Credentials::UserPassword {
        username: "username".into(),
        password: "password".into(),
    };
    pcloud::http::HttpClientBuilder::default()
        .region(region)
        .credentials(creds)
        .timeout(std::time::Duration::from_secs(10))
        .build()
        .unwrap()
}

#[test]
fn creating_new_folder() {
    init();
    let root = tempfile::TempDir::new().unwrap();
    let cache = tempfile::TempDir::new().unwrap();
    let service = pcloud_fuse::service::Service::new(build_client(), cache.path().to_path_buf());
    let fs = pcloud_fuse::fs::PCloudFs::new(service);
    let root_path = root.path().to_path_buf().into_os_string();
    let root_path = root_path.to_str().unwrap().to_string();
    let options = vec![MountOption::AutoUnmount, MountOption::AllowOther];
    let _mount_handle = fuser::spawn_mount2(fs, root_path, &options).unwrap();
    // started and running
    let list_root = mockito::mock(
        "GET",
        "/listfolder?username=username&password=password&folderid=0",
    )
    .expect_at_least(1)
    .with_body(
        r#"{
    "result": 0,
    "metadata": {
        "icon": "folder",
        "id": "d0",
        "modified": "Thu, 19 Sep 2013 07:31:46 +0000",
        "path": "/",
        "thumb": false,
        "created": "Thu, 19 Sep 2013 07:31:46 +0000",
        "folderid": 0,
        "isshared": false,
        "isfolder": true,
        "ismine": true,
        "name": "/",
        "contents": []
    }
}"#,
    )
    .create();
    let create_folder = mockito::mock(
        "GET",
        "/createfolder?username=username&password=password&name=child&folderid=0",
    )
    .expect(1)
    .with_body(
        r#"{
    "result": 0,
    "metadata": {
        "path": "\/child",
        "name": "child",
        "created": "Fri, 23 Jul 2021 19:39:09 +0000",
        "ismine": true,
        "thumb": false,
        "modified": "Fri, 23 Jul 2021 19:39:09 +0000",
        "id": "d000002",
        "isshared": false,
        "icon": "folder",
        "isfolder": true,
        "parentfolderid": 0,
        "folderid": 2
    }
}"#,
    )
    .create();
    std::fs::create_dir(root.path().join("child")).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));
    list_root.assert();
    create_folder.assert();
}

#[test]
fn writing_new_file() {
    init();
    let root = tempfile::TempDir::new().unwrap();
    let cache = tempfile::TempDir::new().unwrap();
    let service = pcloud_fuse::service::Service::new(build_client(), cache.path().to_path_buf());
    let fs = pcloud_fuse::fs::PCloudFs::new(service);
    let root_path = root.path().to_path_buf().into_os_string();
    let root_path = root_path.to_str().unwrap().to_string();
    let options = vec![MountOption::AutoUnmount, MountOption::AllowOther];
    let _mount_handle = fuser::spawn_mount2(fs, root_path, &options).unwrap();
    // started and running
    let list_root = mockito::mock(
        "GET",
        "/listfolder?username=username&password=password&folderid=0",
    )
    .expect_at_least(1)
    .with_body(
        r#"{
    "result": 0,
    "metadata": {
        "icon": "folder",
        "id": "d0",
        "modified": "Thu, 19 Sep 2013 07:31:46 +0000",
        "path": "/",
        "thumb": false,
        "created": "Thu, 19 Sep 2013 07:31:46 +0000",
        "folderid": 0,
        "isshared": false,
        "isfolder": true,
        "ismine": true,
        "name": "/",
        "contents": []
    }
}"#,
    )
    .create();
    let checksum = mockito::mock(
        "GET",
        "/checksumfile?username=username&password=password&fileid=1",
    )
    .expect(1)
    .with_body(
        r#"{
    "result": 0,
    "sha256": "d535d3354f9d36741e311ac0855c5cde1e8e90eae947f320469f17514d182e19",
    "sha1": "5b03ef4fa47ed13f2156ec5395866dadbde4e9dc",
    "metadata": {
        "name": "child.bin",
        "path": "/child.bin",
        "created": "Fri, 12 Aug 2022 20:20:08 +0000",
        "thumb": false,
        "modified": "Fri, 12 Aug 2022 20:20:08 +0000",
        "isfolder": false,
        "fileid": 1,
        "hash": 124557230560423924,
        "category": 0,
        "id": "f000001",
        "isshared": false,
        "ismine": true,
        "size": 10485760,
        "parentfolderid": 0,
        "contenttype": "application\/octet-stream",
        "icon": "file"
    }
}"#,
    )
    .create();
    let upload_file = mockito::mock(
        "POST",
        "/uploadfile?username=username&password=password&folderid=0",
    )
    .expect(1)
    .with_body(
        r#"{
    "result": 0,
    "metadata": [
        {
            "name": "child.bin",
            "created": "Fri, 12 Aug 2022 20:20:08 +0000",
            "thumb": false,
            "modified": "Fri, 12 Aug 2022 20:20:08 +0000",
            "isfolder": false,
            "fileid": 1,
            "hash": 124557230560423924,
            "category": 0,
            "id": "f000001",
            "isshared": false,
            "ismine": true,
            "size": 10485760,
            "parentfolderid": 0,
            "contenttype": "application\/octet-stream",
            "icon": "file"
        }
    ],
    "checksums": [
        {
            "sha1": "42dad410ba75571171543d668696f6123ef70d49",
            "sha256": "2e06bb6b60c13edcef72599bdf44e570bba0c21175ebc3016420e36fd2798c10"
        }
    ],
    "fileids": [1]
}"#,
    )
    .with_status(200)
    .create();
    let mut file = File::create(root.path().join("child.bin")).unwrap();
    writeln!(&mut file, "Hello World!").unwrap();
    drop(file);
    std::thread::sleep(std::time::Duration::from_secs(1));
    list_root.assert();
    checksum.assert();
    upload_file.assert();
}

// #[test]
fn _writing_existing_file() {
    init();
    let root = tempfile::TempDir::new().unwrap();
    let cache = tempfile::TempDir::new().unwrap();
    let service = pcloud_fuse::service::Service::new(build_client(), cache.path().to_path_buf());
    let fs = pcloud_fuse::fs::PCloudFs::new(service);
    let root_path = root.path().to_path_buf().into_os_string();
    let root_path = root_path.to_str().unwrap().to_string();
    let options = vec![MountOption::AutoUnmount, MountOption::AllowOther];
    let _mount_handle = fuser::spawn_mount2(fs, root_path, &options).unwrap();
    // started and running
    let list_root = mockito::mock("GET", "/listfolder")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("username".into(), "username".into()),
            Matcher::UrlEncoded("password".into(), "password".into()),
            Matcher::UrlEncoded("folderid".into(), "0".into()),
        ]))
        .with_body(
            r#"{
    "result": 0,
    "metadata": {
        "icon": "folder",
        "id": "d0",
        "modified": "Thu, 19 Sep 2013 07:31:46 +0000",
        "path": "/",
        "thumb": false,
        "created": "Thu, 19 Sep 2013 07:31:46 +0000",
        "folderid": 0,
        "isshared": false,
        "isfolder": true,
        "ismine": true,
        "name": "/",
        "contents": [
            {
                "name": "child.bin",
                "path": "/child.bin",
                "created": "Fri, 12 Aug 2022 20:20:08 +0000",
                "thumb": false,
                "modified": "Fri, 12 Aug 2022 20:20:08 +0000",
                "isfolder": false,
                "fileid": 1,
                "hash": 124557230560423924,
                "category": 0,
                "id": "f000001",
                "isshared": false,
                "ismine": true,
                "size": 10485760,
                "parentfolderid": 0,
                "contenttype": "application\/octet-stream",
                "icon": "file"
            }
        ]
    }
}"#,
        )
        .with_status(200)
        .create();
    let getfilelink = mockito::mock("GET", "/getfilelink")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("username".into(), "username".into()),
            Matcher::UrlEncoded("password".into(), "password".into()),
            Matcher::UrlEncoded("fileid".into(), "1".into()),
        ]))
        .with_body(format!(
            r#"{{
    "result": 0,
    "dwltag": "yvkNr0TqT6HFAWlVpdnHs5",
    "hash": 17869736033964340520,
    "size": 42,
    "expires": "Sat, 24 Jul 2021 03:18:31 +0000",
    "path": "\/wherever\/child.bin",
    "hosts": ["{}"]
}}"#,
            mockito::server_url()
        ))
        .with_status(200)
        .create();
    let downloadfile = mockito::mock("GET", "/wherever/child.bin")
        .expect(1)
        .with_body("What ")
        .create();
    let upload_file = mockito::mock("POST", "/uploadfile")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("username".into(), "username".into()),
            Matcher::UrlEncoded("password".into(), "password".into()),
            Matcher::UrlEncoded("folderid".into(), "0".into()),
        ]))
        .match_header(
            "content-type",
            Matcher::Regex("multipart/form-data; boundary=.*".to_string()),
        )
        .match_body(Matcher::Any)
        .with_body(
            r#"{
    "result": 0,
    "metadata": [
        {
            "name": "child.bin",
            "created": "Fri, 12 Aug 2022 20:20:08 +0000",
            "thumb": false,
            "modified": "Fri, 12 Aug 2022 20:20:08 +0000",
            "isfolder": false,
            "fileid": 1,
            "hash": 124557230560423924,
            "category": 0,
            "id": "f000001",
            "isshared": false,
            "ismine": true,
            "size": 10485760,
            "parentfolderid": 0,
            "contenttype": "application\/octet-stream",
            "icon": "file"
        }
    ],
    "checksums": [
        {
            "sha1": "42dad410ba75571171543d668696f6123ef70d49",
            "sha256": "2e06bb6b60c13edcef72599bdf44e570bba0c21175ebc3016420e36fd2798c10"
        }
    ],
    "fileids": [1]
}"#,
        )
        .with_status(200)
        .create();
    //
    let mut file = OpenOptions::new()
        .read(false)
        .write(true)
        .append(true)
        .open(root.path().join("child.bin"))
        .unwrap();
    writeln!(&mut file, "The Hell").unwrap();
    file.flush().unwrap();
    drop(file);
    std::thread::sleep(std::time::Duration::from_secs(1));
    list_root.assert();
    getfilelink.assert();
    downloadfile.assert();
    upload_file.assert();
}

#[test]
fn moving_folder() {
    init();
    let root = tempfile::TempDir::new().unwrap();
    let cache = tempfile::TempDir::new().unwrap();
    let service = pcloud_fuse::service::Service::new(build_client(), cache.path().to_path_buf());
    let fs = pcloud_fuse::fs::PCloudFs::new(service);
    let root_path = root.path().to_path_buf().into_os_string();
    let root_path = root_path.to_str().unwrap().to_string();
    let options = vec![MountOption::AutoUnmount, MountOption::AllowOther];
    let _mount_handle = fuser::spawn_mount2(fs, root_path, &options).unwrap();
    // started and running
    let list_root = mockito::mock("GET", "/listfolder")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("username".into(), "username".into()),
            Matcher::UrlEncoded("password".into(), "password".into()),
            Matcher::UrlEncoded("folderid".into(), "0".into()),
        ]))
        .with_body(
            r#"{
    "result": 0,
    "metadata": {
        "icon": "folder",
        "id": "d0",
        "modified": "Thu, 19 Sep 2013 07:31:46 +0000",
        "path": "/",
        "thumb": false,
        "created": "Thu, 19 Sep 2013 07:31:46 +0000",
        "folderid": 0,
        "isshared": false,
        "isfolder": true,
        "ismine": true,
        "name": "/",
        "contents": [
            {
                "path": "/first",
                "name": "first",
                "created": "Fri, 23 Jul 2021 19:39:09 +0000",
                "ismine": true,
                "thumb": false,
                "modified": "Fri, 23 Jul 2021 19:39:09 +0000",
                "id": "d000001",
                "isshared": false,
                "icon": "folder",
                "isfolder": true,
                "parentfolderid": 0,
                "folderid": 1
            }
        ]
    }
}"#,
        )
        .with_status(200)
        .create();
    let rename_folder = mockito::mock("GET", "/renamefolder")
        .match_query(Matcher::AllOf(vec![
            Matcher::UrlEncoded("username".into(), "username".into()),
            Matcher::UrlEncoded("password".into(), "password".into()),
            Matcher::UrlEncoded("folderid".into(), "1".into()),
            Matcher::UrlEncoded("toname".into(), "second".into()),
        ]))
        .with_body(
            r#"{
    "result": 0,
    "metadata": {
        "path": "/second",
        "name": "second",
        "created": "Fri, 23 Jul 2021 19:39:09 +0000",
        "ismine": true,
        "thumb": false,
        "modified": "Fri, 23 Jul 2021 19:39:09 +0000",
        "id": "d000001",
        "isshared": false,
        "icon": "folder",
        "isfolder": true,
        "parentfolderid": 0,
        "folderid": 1
    }
}"#,
        )
        .with_status(200)
        .create();
    //
    std::fs::rename(root.path().join("first"), root.path().join("second")).unwrap();
    std::thread::sleep(std::time::Duration::from_secs(1));
    list_root.assert();
    rename_folder.assert();
}
