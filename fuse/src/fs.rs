use crate::service::{Error, PCloudService};
use fuser::{FileType, Filesystem};
use pcloud::entry::{Entry, File, Folder};
use std::ffi::OsStr;
use std::os::raw::c_int;
use std::time::{Duration, UNIX_EPOCH};

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
    pub fn new(service: PCloudService, _folder_id: usize, _data_dir: String) -> Self {
        Self {
            service,
            fuse_duration: Duration::new(2, 0),
        }
    }
}

impl Filesystem for PCloudFs {
    #[tracing::instrument(skip_all)]
    fn init(
        &mut self,
        _req: &fuser::Request,
        _config: &mut fuser::KernelConfig,
    ) -> Result<(), c_int> {
        Ok(())
    }

    #[tracing::instrument(skip_all)]
    fn destroy(&mut self, _req: &fuser::Request) {}

    #[tracing::instrument(skip(self, _req, reply))]
    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEntry,
    ) {
        let name = if let Some(value) = name.to_str() {
            value
        } else {
            tracing::error!("Path component is not UTF-8");
            reply.error(Error::InvalidArgument.into());
            return;
        };
        let parent = match self.service.get_folder(parent) {
            Ok(value) => value,
            Err(err) => return reply.error(err.into()),
        };
        let entries = parent.contents.clone().unwrap_or_default();
        let entry = entries.iter().find(|item| item.base().name == name);

        if let Some(entry) = entry {
            let attr = create_entry_attrs(entry);
            reply.entry(&self.fuse_duration, &attr, 0);
        } else {
            // file doesn't exist
            reply.error(Error::NotFound.into());
        }
    }

    #[tracing::instrument(skip(self, _req))]
    fn forget(&mut self, _req: &fuser::Request<'_>, ino: u64, nlookup: u64) {}

    #[tracing::instrument(skip(self, _req, reply))]
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

    #[tracing::instrument(skip(self, _req, reply))]
    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        inode: u64,
        _fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        assert!(offset >= 0);
        let folder = match self.service.get_folder(inode) {
            Ok(value) => value,
            Err(err) => return reply.error(err.into()),
        };
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

    #[tracing::instrument(skip(self, _req, reply))]
    fn readdirplus(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        _offset: i64,
        reply: fuser::ReplyDirectoryPlus,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn releasedir(
        &mut self,
        _req: &fuser::Request,
        inode: u64,
        fh: u64,
        _flags: i32,
        reply: fuser::ReplyEmpty,
    ) {
        self.service.release(fh);
        reply.ok();
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn flush(
        &mut self,
        _req: &fuser::Request,
        inode: u64,
        _fh: u64,
        _lock_owner: u64,
        reply: fuser::ReplyEmpty,
    ) {
        reply.ok();
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn release(
        &mut self,
        _req: &fuser::Request,
        inode: u64,
        fh: u64,
        _flags: i32,
        _lock_owner: Option<u64>,
        _flush: bool,
        reply: fuser::ReplyEmpty,
    ) {
        self.service.release(fh);
        reply.ok();
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn access(&mut self, _req: &fuser::Request<'_>, ino: u64, mask: i32, reply: fuser::ReplyEmpty) {
        reply.ok();
    }

    #[tracing::instrument(skip(self, _req, reply))]
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

    #[tracing::instrument(skip(self, _req, reply))]
    fn read(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        size: u32,
        flags: i32,
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

    #[tracing::instrument(skip(self, _req, reply))]
    fn fallocate(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        offset: i64,
        length: i64,
        mode: i32,
        reply: fuser::ReplyEmpty,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn bmap(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        blocksize: u32,
        idx: u64,
        reply: fuser::ReplyBmap,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
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
        let name = match name.to_str() {
            Some(value) => value,
            None => {
                tracing::error!("Path component is not UTF-8");
                reply.error(Error::InvalidArgument.into());
                return;
            }
        };
        let handle = match self.service.create_file(parent_id, name) {
            Ok(handle) => handle,
            Err(err) => return reply.error(err.into()),
        };
        let parent = match self.service.fetch_folder(parent_id) {
            Ok(folder) => folder,
            Err(err) => return reply.error(err.into()),
        };
        let children = parent.contents.unwrap_or_default();
        let file = children
            .into_iter()
            .filter_map(|item| item.as_file())
            .find(|item| item.base.name == name);
        if let Some(file) = file {
            let file_attr = create_file_attrs(&file);

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

    #[tracing::instrument(skip(self, _req, data, reply))]
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

    #[tracing::instrument(skip(self, _req, reply))]
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

    #[tracing::instrument(skip(self, _req, reply))]
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

    #[tracing::instrument(skip(self, _req, reply))]
    fn setlk(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
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

    #[tracing::instrument(skip(self, _req, reply))]
    fn getlk(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        _lock_owner: u64,
        _start: u64,
        _end: u64,
        _typ: i32,
        _pid: u32,
        reply: fuser::ReplyLock,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn copy_file_range(
        &mut self,
        _req: &fuser::Request<'_>,
        ino_in: u64,
        fh_in: u64,
        offset_in: i64,
        ino_out: u64,
        fh_out: u64,
        offset_out: i64,
        _len: u64,
        _flags: u32,
        reply: fuser::ReplyWrite,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn fsync(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        _datasync: bool,
        reply: fuser::ReplyEmpty,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn fsyncdir(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        _datasync: bool,
        reply: fuser::ReplyEmpty,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn getxattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        name: &OsStr,
        _size: u32,
        reply: fuser::ReplyXattr,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn setxattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        name: &OsStr,
        _value: &[u8],
        _flags: i32,
        _position: u32,
        reply: fuser::ReplyEmpty,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn removexattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        name: &OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
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
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn link(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        newparent: u64,
        _newname: &OsStr,
        reply: fuser::ReplyEntry,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn listxattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _size: u32,
        reply: fuser::ReplyXattr,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn lseek(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        _offset: i64,
        _whence: i32,
        reply: fuser::ReplyLseek,
    ) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn mkdir(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        reply: fuser::ReplyEntry,
    ) {
        let name = match name.to_str() {
            Some(value) => value,
            None => {
                tracing::error!("Path component is not UTF-8");
                reply.error(Error::InvalidArgument.into());
                return;
            }
        };
        match self.service.create_folder(parent, name) {
            Ok(folder) => {
                let attrs = create_folder_attrs(&folder);
                reply.entry(&self.fuse_duration, &attrs, folder.folder_id as u64)
            }
            Err(err) => reply.error(err.into()),
        };
    }

    #[tracing::instrument(skip(self, _req, reply))]
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
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn readlink(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyData) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
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
        let name = match name.to_str() {
            Some(value) => value,
            None => {
                tracing::error!("Path component is not UTF-8");
                reply.error(Error::InvalidArgument.into());
                return;
            }
        };
        let new_name = match new_name.to_str() {
            Some(value) => value,
            None => {
                tracing::error!("Path component is not UTF-8");
                reply.error(Error::InvalidArgument.into());
                return;
            }
        };
        match self.service.rename(parent, name, new_parent, new_name) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err.into()),
        }
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn rmdir(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        let name = match name.to_str() {
            Some(value) => value,
            None => {
                tracing::error!("Path component is not UTF-8");
                reply.error(Error::InvalidArgument.into());
                return;
            }
        };
        match self.service.remove_folder(parent, name) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err.into()),
        }
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn statfs(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyStatfs) {
        reply.error(Error::NotImplemented.into());
    }

    #[tracing::instrument(skip(self, _req, reply))]
    fn unlink(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        let name = match name.to_str() {
            Some(value) => value,
            None => {
                tracing::error!("Path component is not UTF-8");
                reply.error(Error::InvalidArgument.into());
                return;
            }
        };
        match self.service.remove_file(parent, name) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err.into()),
        };
    }
}
