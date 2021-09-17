pub mod close;
pub mod open;
pub mod pread;
pub mod pwrite;
pub mod read;
pub mod seek;
pub mod size;

#[cfg(test)]
mod tests {
    use crate::binary::BinaryClient;
    use crate::credentials::Credentials;
    use crate::region::Region;

    #[test]
    fn reading() {
        let creds = Credentials::from_env();
        let mut client = BinaryClient::new(Region::eu(), creds).unwrap();
        let params = super::open::Params::new(0).identifier(5837100991.into());
        let fd = client.file_open(&params).unwrap();
        let params = super::read::Params::new(fd, 8);
        let data = client.file_read(&params).unwrap();
        assert_eq!(data, [255, 216, 255, 224, 0, 16, 74, 70]);
        let size = client.file_size(fd).unwrap();
        assert_eq!(size.offset, 8);
        assert_eq!(size.size, 467128);
        let params = super::seek::Params::new(fd, std::io::SeekFrom::Start(0));
        client.file_seek(&params).unwrap();
        client.file_close(fd).unwrap();
    }
}
