use crate::service::{Error, Service};
use fuser::Filesystem;
use std::ffi::OsStr;
use std::time::Duration;

macro_rules! parse_str {
    ($value:ident, $reply:ident) => {
        if let Some(value) = $value.to_str() {
            value
        } else {
            tracing::error!("Path component is not UTF-8");
            return $reply.error(Error::InvalidArgument.into_code());
        }
    };
}

macro_rules! resolve {
    ($value:expr, $reply:ident) => {
        match $value {
            Ok(value) => value,
            Err(err) => return $reply.error(err.into_code()),
        }
    };
}

pub struct PCloudFs {
    service: Service,
    fuse_duration: Duration,
}

impl PCloudFs {
    pub fn new(service: Service) -> Self {
        Self {
            service,
            fuse_duration: Duration::new(2, 0),
        }
    }
}

impl Filesystem for PCloudFs {
    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEntry,
    ) {
        tracing::debug!("lookup parent={parent} name={:?}", name);
        let name = parse_str!(name, reply);
        match self.service.read_file_in_directory(parent, name) {
            Ok(attrs) => reply.entry(&self.fuse_duration, &attrs, 0),
            Err(err) => reply.error(err.into_code()),
        }
    }

    fn opendir(&mut self, _req: &fuser::Request, inode: u64, flags: i32, reply: fuser::ReplyOpen) {
        tracing::debug!("opendir inode={inode}");
        match self.service.open_directory(inode, flags) {
            Ok(fh) => reply.opened(fh, 0),
            Err(err) => reply.error(err.into_code()),
        };
    }

    fn releasedir(
        &mut self,
        _req: &fuser::Request,
        inode: u64,
        fh: u64,
        _flags: i32,
        reply: fuser::ReplyEmpty,
    ) {
        tracing::debug!("releasedir inode={inode} handler={fh}");
        match self.service.close_directory(inode, fh) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err.into_code()),
        };
    }

    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        tracing::debug!("getattr inode={ino}");
        match self.service.get_attributes(ino) {
            Ok(res) => reply.attr(&self.fuse_duration, &res),
            Err(err) => reply.error(err.into_code()),
        }
    }

    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        inode: u64,
        fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        tracing::debug!("readdir inode={inode} fh={fh}");
        assert!(offset >= 0);
        let children = resolve!(self.service.read_directory(inode), reply);

        for item in children.into_iter().skip(offset as usize) {
            let buffer_full = reply.add(
                item.id,
                item.position,
                item.file_type,
                OsStr::new(&item.name),
            );

            if buffer_full {
                break;
            }
        }

        reply.ok();
    }

    fn open(&mut self, _req: &fuser::Request<'_>, ino: u64, _flags: i32, reply: fuser::ReplyOpen) {
        tracing::debug!("open inode={ino}");
        match self.service.open_file(ino) {
            // check https://man7.org/linux/man-pages/man4/fuse.4.html for the return flags
            Ok(res) => reply.opened(res as u64, 0),
            Err(err) => reply.error(err.into_code()),
        };
    }

    fn release(
        &mut self,
        _req: &fuser::Request,
        inode: u64,
        fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        flush: bool,
        reply: fuser::ReplyEmpty,
    ) {
        tracing::debug!("release inode={inode} fh={fh} flush={flush}");
        match self.service.close_file(inode, fh, flush) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err.into_code()),
        };
    }

    fn read(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyData,
    ) {
        tracing::debug!("read inode={ino} fh={fh} offset={offset} size={size}");
        match self.service.read_file(ino, fh, offset, size) {
            Ok(res) => reply.data(&res),
            Err(err) => reply.error(err.into_code()),
        };
    }
}
