use crate::service::{Error, Service};
use fuser::Filesystem;
use std::ffi::OsStr;
use std::time::Duration;
use tracing::instrument;

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
    #[instrument(skip(self, _req, reply))]
    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEntry,
    ) {
        let name = parse_str!(name, reply);
        match self.service.read_file_in_directory(parent, name) {
            Ok(attrs) => reply.entry(&self.fuse_duration, &attrs, 0),
            Err(err) => reply.error(err.into_code()),
        }
    }

    #[instrument(skip(self, _req, reply))]
    fn opendir(&mut self, _req: &fuser::Request, inode: u64, _flags: i32, reply: fuser::ReplyOpen) {
        match self.service.open_directory(inode) {
            Ok(fh) => reply.opened(fh, 0),
            Err(err) => reply.error(err.into_code()),
        };
    }

    #[instrument(skip(self, _req, _flags, reply))]
    fn releasedir(
        &mut self,
        _req: &fuser::Request,
        inode: u64,
        fh: u64,
        _flags: i32,
        reply: fuser::ReplyEmpty,
    ) {
        match self.service.close_directory(inode, fh) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err.into_code()),
        };
    }

    #[instrument(skip(self, _req, reply))]
    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        match self.service.get_attributes(ino) {
            Ok(res) => reply.attr(&self.fuse_duration, &res),
            Err(err) => reply.error(err.into_code()),
        }
    }

    #[instrument(skip(self, _req, reply))]
    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        inode: u64,
        fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
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

    #[instrument(skip(self, _req, flags, reply))]
    fn open(&mut self, _req: &fuser::Request<'_>, ino: u64, flags: i32, reply: fuser::ReplyOpen) {
        match self.service.open_file(ino, flags) {
            // check https://man7.org/linux/man-pages/man4/fuse.4.html for the return flags
            Ok(res) => reply.opened(res as u64, 0),
            Err(err) => reply.error(err.into_code()),
        };
    }

    #[instrument(skip(self, _req, _flags, _lock_owner, reply))]
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
        match self.service.close_file(inode, fh, flush) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err.into_code()),
        };
    }

    #[instrument(skip(self, _req, _flags, _lock_owner, reply))]
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
        match self.service.read_file(ino, fh, offset, size) {
            Ok(res) => reply.data(&res),
            Err(err) => reply.error(err.into_code()),
        };
    }

    #[instrument(skip(self, _req, _mode, _umask, reply))]
    fn mkdir(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        reply: fuser::ReplyEntry,
    ) {
        let name = parse_str!(name, reply);
        match self.service.create_directory(parent, name) {
            Ok(res) => reply.entry(&self.fuse_duration, &res, 0),
            Err(err) => reply.error(err.into_code()),
        };
    }

    #[instrument(skip(self, _req, reply))]
    fn rmdir(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        let name = parse_str!(name, reply);
        match self.service.remove_directory(parent, name) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err.into_code()),
        };
    }

    #[instrument(skip(self, _req, _mode, _umask, _flags, reply))]
    fn create(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        _flags: i32,
        reply: fuser::ReplyCreate,
    ) {
        let name = parse_str!(name, reply);
        match self.service.create_file(parent, name) {
            Ok((file, fh)) => reply.created(&self.fuse_duration, &file, 0, fh, 0),
            Err(err) => reply.error(err.into_code()),
        };
    }

    #[instrument(skip(self, _req, data, _write_flags, _flags, _lock_owner, reply))]
    fn write(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
        _write_flags: u32,
        _flags: i32,
        _lock_owner: Option<u64>,
        reply: fuser::ReplyWrite,
    ) {
        match self.service.write_file(ino, fh, offset, data) {
            Ok(count) => reply.written(count),
            Err(err) => reply.error(err.into_code()),
        };
    }

    #[instrument(skip(self, _req, reply))]
    fn unlink(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        let name = parse_str!(name, reply);
        match self.service.remove_file(parent, name) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err.into_code()),
        };
    }

    #[instrument(skip(self, _req, _flags, reply))]
    fn rename(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        newparent: u64,
        newname: &OsStr,
        _flags: u32,
        reply: fuser::ReplyEmpty,
    ) {
        let name = parse_str!(name, reply);
        let newname = parse_str!(newname, reply);
        match self.service.move_entry(parent, name, newparent, newname) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err.into_code()),
        }
    }

    #[instrument(skip(self, _req, _lock_owner, reply))]
    fn flush(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        _lock_owner: u64,
        reply: fuser::ReplyEmpty,
    ) {
        reply.ok();
    }
}
