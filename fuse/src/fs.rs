use crate::service::{Error, PCloudService};
use fuser::{FileType, Filesystem};
use pcloud::entry::{Entry, File, Folder};
use std::ffi::OsStr;
use std::os::raw::c_int;
use std::time::{Duration, UNIX_EPOCH};

macro_rules! parse_str {
    ($value:ident, $reply:ident) => {
        if let Some(value) = $value.to_str() {
            value
        } else {
            tracing::error!("Path component is not UTF-8");
            return $reply.error(Error::InvalidArgument.into());
        }
    };
}

macro_rules! resolve {
    ($value:expr, $reply:ident) => {
        match $value {
            Ok(value) => value,
            Err(err) => return $reply.error(err.into()),
        }
    };
}

fn create_file_attrs(file: &File) -> fuser::FileAttr {
    fuser::FileAttr {
        ino: (file.file_id + 1) as u64,
        size: file.size.unwrap_or(0) as u64,
        blocks: file.size.unwrap_or(0) as u64,
        blksize: 1,
        atime: UNIX_EPOCH,
        mtime: file.base.modified.into(),
        ctime: file.base.modified.into(),
        crtime: file.base.created.into(),
        kind: fuser::FileType::RegularFile,
        perm: 0o666,
        nlink: 0,
        uid: 1000,
        gid: 1000,
        rdev: 0,
        flags: 0,
    }
}

fn create_folder_attrs(folder: &Folder) -> fuser::FileAttr {
    fuser::FileAttr {
        ino: (folder.folder_id + 1) as u64,
        size: 1,
        blocks: 1,
        blksize: 1,
        atime: UNIX_EPOCH,
        mtime: folder.base.modified.into(),
        ctime: folder.base.modified.into(),
        crtime: folder.base.created.into(),
        kind: fuser::FileType::Directory,
        perm: 0o777,
        nlink: 0,
        uid: 1000,
        gid: 1000,
        rdev: 0,
        flags: 0,
    }
}

fn create_entry_attrs(entry: &Entry) -> fuser::FileAttr {
    match entry {
        Entry::File(file) => create_file_attrs(file),
        Entry::Folder(folder) => create_folder_attrs(folder),
    }
}

pub struct PCloudFs {
    service: PCloudService,
    fuse_duration: Duration,
}

impl PCloudFs {
    pub fn new(service: PCloudService) -> Self {
        Self {
            service,
            fuse_duration: Duration::new(2, 0),
        }
    }
}

impl Filesystem for PCloudFs {
    fn init(
        &mut self,
        _req: &fuser::Request,
        _config: &mut fuser::KernelConfig,
    ) -> Result<(), c_int> {
        tracing::trace!("init");
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEntry,
    ) {
        let name = parse_str!(name, reply);
        let parent = resolve!(self.service.get_folder(parent), reply);
        let entries = parent.contents.unwrap_or_default();
        let entry = entries.iter().find(|item| item.base().name == name);

        if let Some(entry) = entry {
            let attr = create_entry_attrs(entry);
            reply.entry(&self.fuse_duration, &attr, 0);
        } else {
            // file doesn't exist
            reply.error(Error::NotFound.into());
        }
    }

    fn forget(&mut self, _req: &fuser::Request<'_>, ino: u64, _nlookup: u64) {
        tracing::trace!("forget ino={}", ino);
    }

    #[tracing::instrument(skip_all)]
    fn opendir(&mut self, _req: &fuser::Request, inode: u64, flags: i32, reply: fuser::ReplyOpen) {
        match self.service.open_folder(inode, flags) {
            Ok(fh) => {
                reply.opened(fh, 0);
            }
            Err(err) => {
                reply.error(err.into());
            }
        };
    }

    #[tracing::instrument(skip_all)]
    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        inode: u64,
        _fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        assert!(offset >= 0);
        let folder = resolve!(self.service.get_folder(inode), reply);
        let children = folder.contents.unwrap_or_default();
        for (index, entry) in children.iter().skip(offset as usize).enumerate() {
            let buffer_full = match entry {
                Entry::File(file) => reply.add(
                    file.file_id as u64 + 1,
                    offset + index as i64 + 1,
                    FileType::RegularFile,
                    OsStr::new(&file.base.name),
                ),
                Entry::Folder(folder) => reply.add(
                    folder.folder_id as u64 + 1,
                    offset + index as i64 + 1,
                    FileType::Directory,
                    OsStr::new(&folder.base.name),
                ),
            };

            if buffer_full {
                break;
            }
        }

        reply.ok();
    }

    fn readdirplus(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        _offset: i64,
        reply: fuser::ReplyDirectoryPlus,
    ) {
        tracing::trace!("read dir plus ino={} fh={}", ino, fh);
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip_all)]
    fn releasedir(
        &mut self,
        _req: &fuser::Request,
        _inode: u64,
        fh: u64,
        _flags: i32,
        reply: fuser::ReplyEmpty,
    ) {
        self.service.release(fh);
        reply.ok();
    }

    fn flush(
        &mut self,
        _req: &fuser::Request,
        ino: u64,
        fh: u64,
        _lock_owner: u64,
        reply: fuser::ReplyEmpty,
    ) {
        tracing::trace!("flush ino={} fh={}", ino, fh);
        reply.ok();
    }

    #[tracing::instrument(skip_all)]
    fn release(
        &mut self,
        _req: &fuser::Request,
        _inode: u64,
        fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: fuser::ReplyEmpty,
    ) {
        self.service.release(fh);
        reply.ok();
    }

    fn access(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _mask: i32,
        reply: fuser::ReplyEmpty,
    ) {
        tracing::trace!("access ino={}", ino);
        reply.ok();
    }

    #[tracing::instrument(skip_all)]
    fn open(&mut self, _req: &fuser::Request<'_>, ino: u64, flags: i32, reply: fuser::ReplyOpen) {
        match self.service.open_file(ino, flags) {
            Ok(res) => {
                reply.opened(res as u64, flags as u32);
            }
            Err(err) => {
                reply.error(err.into());
            }
        }
    }

    #[tracing::instrument(skip_all)]
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
            Ok(res) => {
                reply.data(&res);
            }
            Err(err) => {
                reply.error(err.into());
            }
        };
    }

    fn fallocate(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        _offset: i64,
        _length: i64,
        _mode: i32,
        reply: fuser::ReplyEmpty,
    ) {
        tracing::trace!("fallocate ino={} fh={}", ino, fh);
        reply.error(Error::NotImplemented.into());
    }

    fn bmap(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _blocksize: u32,
        _idx: u64,
        reply: fuser::ReplyBmap,
    ) {
        tracing::trace!("bmap ino={}", ino);
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip_all)]
    fn create(
        &mut self,
        _req: &fuser::Request<'_>,
        parent_id: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        _flags: i32,
        reply: fuser::ReplyCreate,
    ) {
        let name = parse_str!(name, reply);
        let handle = resolve!(self.service.create_file(parent_id, name), reply);
        let parent = resolve!(self.service.fetch_folder(parent_id), reply);
        if let Some(file) = parent.find_file(name) {
            let file_attr = create_file_attrs(file);

            reply.created(
                &self.fuse_duration,
                &file_attr,
                file.file_id as u64,
                handle,
                0o666,
            )
        } else {
            tracing::error!("Unable to find the created file");
            reply.error(Error::NotFound.into());
        };
    }

    #[tracing::instrument(skip_all)]
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
            Ok(size) => reply.written(size as u32),
            Err(err) => reply.error(err.into()),
        };
    }

    #[tracing::instrument(skip_all)]
    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        match self.service.get_folder(ino) {
            Ok(folder) => {
                reply.attr(&self.fuse_duration, &create_folder_attrs(&folder));
            }
            Err(_err) => match self.service.get_file(ino) {
                Ok(result) => {
                    reply.attr(&self.fuse_duration, &create_file_attrs(&result));
                }
                Err(err) => {
                    reply.error(err.into());
                }
            },
        };
    }

    #[tracing::instrument(skip_all)]
    fn setattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _mode: Option<u32>,
        _uid: Option<u32>,
        _gid: Option<u32>,
        _size: Option<u64>,
        _atime: Option<fuser::TimeOrNow>,
        _mtime: Option<fuser::TimeOrNow>,
        _ctime: Option<std::time::SystemTime>,
        _fh: Option<u64>,
        _crtime: Option<std::time::SystemTime>,
        _chgtime: Option<std::time::SystemTime>,
        _bkuptime: Option<std::time::SystemTime>,
        _flags: Option<u32>,
        reply: fuser::ReplyAttr,
    ) {
        match self.service.get_file(ino) {
            Ok(result) => reply.attr(&self.fuse_duration, &create_file_attrs(&result)),
            Err(err) => reply.error(err.into()),
        }
    }

    fn setlk(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _fh: u64,
        _lock_owner: u64,
        _start: u64,
        _end: u64,
        _typ: i32,
        _pid: u32,
        _sleep: bool,
        reply: fuser::ReplyEmpty,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    fn getlk(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _fh: u64,
        _lock_owner: u64,
        _start: u64,
        _end: u64,
        _typ: i32,
        _pid: u32,
        reply: fuser::ReplyLock,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    fn copy_file_range(
        &mut self,
        _req: &fuser::Request<'_>,
        ino_in: u64,
        fh_in: u64,
        _offset_in: i64,
        ino_out: u64,
        fh_out: u64,
        _offset_out: i64,
        _len: u64,
        _flags: u32,
        reply: fuser::ReplyWrite,
    ) {
        tracing::trace!(
            "copy file range ino_in={} fh_in={} ino_out={} fh_out={}",
            ino_in,
            fh_in,
            ino_out,
            fh_out
        );
        reply.error(Error::NotImplemented.into());
    }

    fn fsync(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        _datasync: bool,
        reply: fuser::ReplyEmpty,
    ) {
        tracing::trace!("fsync ino={} fh={}", ino, fh);
        reply.error(Error::NotImplemented.into());
    }

    fn fsyncdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        _datasync: bool,
        reply: fuser::ReplyEmpty,
    ) {
        tracing::trace!("fsyncdir ino={} fh={}", ino, fh);
        reply.error(Error::NotImplemented.into());
    }

    fn getxattr(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _name: &OsStr,
        _size: u32,
        reply: fuser::ReplyXattr,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    fn setxattr(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _name: &OsStr,
        _value: &[u8],
        _flags: i32,
        _position: u32,
        reply: fuser::ReplyEmpty,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    fn removexattr(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _name: &OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    fn ioctl(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        _flags: u32,
        _cmd: u32,
        _in_data: &[u8],
        _out_size: u32,
        reply: fuser::ReplyIoctl,
    ) {
        tracing::trace!("ioctl ino={} fh={}", ino, fh);
        reply.error(Error::NotImplemented.into());
    }

    fn link(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _newparent: u64,
        _newname: &OsStr,
        reply: fuser::ReplyEntry,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    fn listxattr(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _size: u32,
        reply: fuser::ReplyXattr,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    fn lseek(
        &mut self,
        _req: &fuser::Request<'_>,
        _ino: u64,
        _fh: u64,
        _offset: i64,
        _whence: i32,
        reply: fuser::ReplyLseek,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip_all)]
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
        match self.service.create_folder(parent, name) {
            Ok(folder) => {
                let attrs = create_folder_attrs(&folder);
                reply.entry(&self.fuse_duration, &attrs, folder.folder_id as u64)
            }
            Err(err) => reply.error(err.into()),
        };
    }

    fn mknod(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        _rdev: u32,
        reply: fuser::ReplyEntry,
    ) {
        tracing::trace!("mknod parent={} name={:?}", parent, name);
        reply.error(Error::NotImplemented.into());
    }

    fn readlink(&mut self, _req: &fuser::Request<'_>, _ino: u64, reply: fuser::ReplyData) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip_all)]
    fn rename(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        new_parent: u64,
        new_name: &OsStr,
        _flags: u32,
        reply: fuser::ReplyEmpty,
    ) {
        let name = parse_str!(name, reply);
        let new_name = parse_str!(new_name, reply);
        match self.service.rename(parent, name, new_parent, new_name) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err.into()),
        }
    }

    #[tracing::instrument(skip_all)]
    fn rmdir(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        let name = parse_str!(name, reply);
        match self.service.remove_folder(parent, name) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err.into()),
        }
    }

    fn statfs(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyStatfs) {
        tracing::trace!("statfs ino={}", ino);
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip_all)]
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
            Err(err) => reply.error(err.into()),
        };
    }
}
