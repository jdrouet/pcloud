use pcloud::binary::BinaryClient;
use pcloud::credentials::Credentials;
use pcloud::http::HttpClient;
use pcloud::region::Region;

async fn http_download_file() -> (u64, Vec<u8>) {
    let client = HttpClient::from_env();
    let params = pcloud::folder::list::Params::new(0);
    let folder = client.list_folder(&params).await.unwrap();
    let file = folder.find_file("landscape.jpeg").unwrap();
    let mut data = Vec::new();
    client.download_file(file.file_id, &mut data).await.unwrap();
    (file.file_id, data)
}

#[tokio::test]
async fn downloading_with_read() {
    let (file_id, expected) = http_download_file().await;
    //
    let creds = Credentials::from_env();
    let mut client = BinaryClient::new(creds, Region::eu()).unwrap();
    //
    let params = pcloud::fileops::open::Params::new(0).identifier(file_id.into());
    let fd = client.file_open(&params).unwrap();
    // size requested by fuse
    let loop_size = 64 * 1024;
    let mut result: Vec<u8> = Vec::new();
    let mut offset = 0;
    while result.len() < expected.len() {
        let params = pcloud::fileops::pread::Params::new(fd, loop_size, offset);
        let data = client.file_pread(&params).unwrap();
        offset += data.len();
        result.extend(&data);
    }
    client.file_close(fd).unwrap();
    assert_eq!(result, expected);
}
