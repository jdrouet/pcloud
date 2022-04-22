#[cfg(feature = "protected")]
mod protected {
    use pcloud::binary::BinaryClientBuilder;
    use pcloud::credentials::Credentials;
    use pcloud::http::HttpClientBuilder;
    use pcloud::region::Region;

    async fn http_download_file() -> (u64, Vec<u8>) {
        use pcloud::prelude::HttpCommand;

        let client = HttpClientBuilder::from_env().build().unwrap();
        let folder = pcloud::folder::list::FolderListCommand::new(0.into())
            .execute(&client)
            .await
            .unwrap();
        let file = folder.find_file("landscape.jpeg").unwrap();
        let mut data = Vec::new();
        pcloud::file::download::FileDownloadCommand::new(file.file_id.into(), &mut data)
            .execute(&client)
            .await
            .unwrap();
        (file.file_id, data)
    }

    #[tokio::test]
    async fn downloading_with_read() {
        use pcloud::prelude::BinaryCommand;

        let (file_id, expected) = http_download_file().await;
        //
        let creds = Credentials::from_env().unwrap();
        let mut client = BinaryClientBuilder::default()
            .set_credentials(creds)
            .set_region(Region::eu())
            .build()
            .unwrap();
        //
        let fd = pcloud::fileops::open::FileOpenCommand::new(0)
            .identifier(file_id.into())
            .execute(&mut client)
            .unwrap();
        // size requested by fuse
        let loop_size = 64 * 1024;
        let mut result: Vec<u8> = Vec::new();
        let mut offset = 0;
        while result.len() < expected.len() {
            let data = pcloud::fileops::pread::FilePReadCommand::new(fd, loop_size, offset)
                .execute(&mut client)
                .unwrap();
            offset += data.len();
            result.extend(&data);
        }
        pcloud::fileops::close::FileCloseCommand::new(fd)
            .execute(&mut client)
            .unwrap();
        assert_eq!(result, expected);
    }
}
