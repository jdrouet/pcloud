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
    fn init(
        &mut self,
        _req: &fuser::Request,
        _config: &mut fuser::KernelConfig,
    ) -> Result<(), c_int> {
        log::debug!("init()");
        Ok(())
    }

    fn destroy(&mut self, _req: &fuser::Request) {
        log::debug!("destroy()");
    }

    fn lookup(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEntry,
    ) {
        log::debug!("lookup() parent={} name={:?}", parent, name);
        let name = if let Some(value) = name.to_str() {
            value
        } else {
            log::error!("Path component is not UTF-8");
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

    fn forget(&mut self, _req: &fuser::Request<'_>, ino: u64, nlookup: u64) {
        log::debug!("forget() ino={}, nlookup={}", ino, nlookup);
    }

    fn opendir(&mut self, _req: &fuser::Request, inode: u64, flags: i32, reply: fuser::ReplyOpen) {
        log::debug!("opendir() ino={}", inode);
        match self.service.open_folder(inode, flags) {
            Ok(fh) => {
                reply.opened(fh, 0);
            }
            Err(err) => {
                reply.error(err.into());
            }
        };
    }

    fn readdir(
        &mut self,
        _req: &fuser::Request<'_>,
        inode: u64,
        _fh: u64,
        offset: i64,
        mut reply: fuser::ReplyDirectory,
    ) {
        log::debug!("readdir() ino={}", inode);
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

    fn readdirplus(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        _offset: i64,
        reply: fuser::ReplyDirectoryPlus,
    ) {
        log::warn!("readdirplus() ino={} fh={}", ino, fh);
        reply.error(Error::NotImplemented.into());
    }

    fn releasedir(
        &mut self,
        _req: &fuser::Request,
        inode: u64,
        fh: u64,
        _flags: i32,
        reply: fuser::ReplyEmpty,
    ) {
        log::debug!("releasedir() ino={} fh={}", inode, fh);
        self.service.release(fh);
        reply.ok();
    }

    fn flush(
        &mut self,
        _req: &fuser::Request,
        inode: u64,
        _fh: u64,
        _lock_owner: u64,
        reply: fuser::ReplyEmpty,
    ) {
        log::debug!("flush() ino={}", inode);
        reply.ok();
    }

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
        log::debug!("release() ino={} fh={}", inode, fh);
        self.service.release(fh);
        reply.ok();
    }

    fn access(&mut self, _req: &fuser::Request<'_>, ino: u64, mask: i32, reply: fuser::ReplyEmpty) {
        log::warn!("access() ino={} mask={}", ino, mask);
        reply.ok();
    }

    fn open(&mut self, _req: &fuser::Request<'_>, ino: u64, flags: i32, reply: fuser::ReplyOpen) {
        log::debug!("open() ino={} flags={}", ino, flags);
        match self.service.open_file(ino, flags) {
            Ok(res) => {
                reply.opened(res as u64, flags as u32);
            }
            Err(err) => {
                reply.error(err.into());
            }
        }
    }

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
        log::debug!(
            "read() ino={} fh={} offset={} size={} flags={}",
            ino,
            fh,
            offset,
            size,
            flags
        );
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
        offset: i64,
        length: i64,
        mode: i32,
        reply: fuser::ReplyEmpty,
    ) {
        log::warn!(
            "fallocate() ino={} fh={} offset={} length={} mode={}",
            ino,
            fh,
            offset,
            length,
            mode
        );
        reply.error(Error::NotImplemented.into());
    }

    fn bmap(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        blocksize: u32,
        idx: u64,
        reply: fuser::ReplyBmap,
    ) {
        log::warn!("bmap() ino={} blocksize={} idx={}", ino, blocksize, idx);
        reply.error(Error::NotImplemented.into());
    }

    fn create(
        &mut self,
        _req: &fuser::Request<'_>,
        parent_id: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: fuser::ReplyCreate,
    ) {
        log::debug!(
            "create() parent={} name={:?} mode={} umask={} flags={}",
            parent_id,
            name,
            mode,
            umask,
            flags
        );
        let name = match name.to_str() {
            Some(value) => value,
            None => {
                log::error!("Path component is not UTF-8");
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
            log::error!("Unable to find the created file");
            reply.error(Error::NotFound.into());
        };
    }

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
        log::debug!(
            "write() ino={} fh={:?} offset={} size={}",
            ino,
            fh,
            offset,
            data.len()
        );
        match self.service.write_file(ino, fh, offset, data) {
            Ok(size) => reply.written(size as u32),
            Err(err) => reply.error(err.into()),
        };
    }

    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        log::debug!("getattr() ino={}", ino);
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
        log::debug!("setattr() ino={}", ino);
        match self.service.get_file(ino) {
            Ok(result) => reply.attr(&self.fuse_duration, &create_file_attrs(&result)),
            Err(err) => reply.error(err.into()),
        }
    }

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
        log::warn!("setlk() ino={} fh={}", ino, fh);
        reply.error(Error::NotImplemented.into());
    }

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
        log::warn!("getlk() ino={} fh={}", ino, fh);
        reply.error(Error::NotImplemented.into());
    }

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
        log::warn!(
            "copy_file_range() ino_in={} fh_in={} offset_in={} ino_out={} fh_out={} offset_out={}",
            ino_in,
            fh_in,
            offset_in,
            ino_out,
            fh_out,
            offset_out,
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
        log::warn!("fsync() ino={} fh={}", ino, fh);
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
        log::warn!("fsyncdir() ino={} fh={}", ino, fh);
        reply.error(Error::NotImplemented.into());
    }

    fn getxattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        name: &OsStr,
        _size: u32,
        reply: fuser::ReplyXattr,
    ) {
        log::warn!("getxattr() ino={} name={:?}", ino, name);
        reply.error(Error::NotImplemented.into());
    }

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
        log::warn!("setxattr() ino={} name={:?}", ino, name);
        reply.error(Error::NotImplemented.into());
    }

    fn removexattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        name: &OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        log::warn!("removexattr() ino={} name={:?}", ino, name);
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
        log::warn!("ioctl() ino={} fh={:?}", ino, fh);
        reply.error(Error::NotImplemented.into());
    }

    fn link(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        newparent: u64,
        _newname: &OsStr,
        reply: fuser::ReplyEntry,
    ) {
        log::warn!("link() ino={} new_parent={}", ino, newparent);
        reply.error(Error::NotImplemented.into());
    }

    fn listxattr(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        _size: u32,
        reply: fuser::ReplyXattr,
    ) {
        log::warn!("listxattr() ino={}", ino);
        reply.error(Error::NotImplemented.into());
    }

    fn lseek(
        &mut self,
        _req: &fuser::Request<'_>,
        ino: u64,
        fh: u64,
        _offset: i64,
        _whence: i32,
        reply: fuser::ReplyLseek,
    ) {
        log::warn!("lseek() ino={} fh={}", ino, fh);
        reply.error(Error::NotImplemented.into());
    }

    fn mkdir(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        _mode: u32,
        _umask: u32,
        reply: fuser::ReplyEntry,
    ) {
        log::debug!("mkdir() parent={} name={:?}", parent, name);
        let name = match name.to_str() {
            Some(value) => value,
            None => {
                log::error!("Path component is not UTF-8");
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
        log::warn!("mknod() parent={} name={:?}", parent, name);
        reply.error(Error::NotImplemented.into());
    }

    fn readlink(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyData) {
        log::warn!("readlink() ino={}", ino);
        reply.error(Error::NotImplemented.into());
    }

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
        log::debug!("rename() parent={} name={:?}", parent, name);
        let name = match name.to_str() {
            Some(value) => value,
            None => {
                log::error!("Path component is not UTF-8");
                reply.error(Error::InvalidArgument.into());
                return;
            }
        };
        let new_name = match new_name.to_str() {
            Some(value) => value,
            None => {
                log::error!("Path component is not UTF-8");
                reply.error(Error::InvalidArgument.into());
                return;
            }
        };
        match self.service.rename(parent, name, new_parent, new_name) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err.into()),
        }
    }

    fn rmdir(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        log::debug!("rmdir() parent={} name={:?}", parent, name);
        let name = match name.to_str() {
            Some(value) => value,
            None => {
                log::error!("Path component is not UTF-8");
                reply.error(Error::InvalidArgument.into());
                return;
            }
        };
        match self.service.remove_folder(parent, name) {
            Ok(_) => reply.ok(),
            Err(err) => reply.error(err.into()),
        }
    }

    fn statfs(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyStatfs) {
        log::warn!("statfs() ino={}", ino);
        reply.error(Error::NotImplemented.into());
    }

    fn unlink(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        reply: fuser::ReplyEmpty,
    ) {
        log::debug!("unlink() parent={} name={:?}", parent, name);
        let name = match name.to_str() {
            Some(value) => value,
            None => {
                log::error!("Path component is not UTF-8");
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
