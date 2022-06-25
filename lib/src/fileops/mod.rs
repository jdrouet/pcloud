pub mod close;
pub mod open;
pub mod pread;
pub mod pwrite;
pub mod read;
pub mod seek;
pub mod size;

const MAX_BLOCK_SIZE: usize = 1024 * 64;

#[cfg(all(test, feature = "protected"))]
mod tests {
    use crate::binary::BinaryClient;
    use crate::credentials::Credentials;
    use crate::prelude::BinaryCommand;
    use crate::region::Region;

    #[test]
    fn reading() {
        let creds = Credentials::from_env().unwrap();
        let mut client = BinaryClient::new(creds, Region::eu()).unwrap();
        let fd = super::open::FileOpenCommand::new(0)
            .identifier(5837100991.into())
            .execute(&mut client)
            .unwrap();
        let data = super::read::FileReadCommand::new(fd, 8)
            .execute(&mut client)
            .unwrap();
        assert_eq!(data, [255, 216, 255, 224, 0, 16, 74, 70]);
        let size = super::size::FileSizeCommand::new(fd)
            .execute(&mut client)
            .unwrap();
        assert_eq!(size.offset, 8);
        assert_eq!(size.size, 467128);
        super::seek::FileSeekCommand::new(fd, std::io::SeekFrom::Start(0))
            .execute(&mut client)
            .unwrap();
        super::close::FileCloseCommand::new(fd)
            .execute(&mut client)
            .unwrap();
    }
}
