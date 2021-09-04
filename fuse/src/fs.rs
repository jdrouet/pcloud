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
        perm: 666,
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
        perm: 666,
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
}

impl PCloudFs {
    pub fn new(service: PCloudService, _folder_id: usize, _data_dir: String) -> Self {
        Self { service }
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
            reply.entry(&Duration::new(2, 0), &attr, 0);
        } else {
            // file doesn't exist
            reply.error(Error::NotFound.into());
        }
    }

    fn forget(&mut self, _req: &fuser::Request<'_>, ino: u64, nlookup: u64) {
        log::debug!("forget() ino={}, nlookup={}", ino, nlookup);
    }

    fn getattr(&mut self, _req: &fuser::Request<'_>, ino: u64, reply: fuser::ReplyAttr) {
        log::debug!("getattr() ino={}", ino);
        match self.service.get_folder(ino) {
            Ok(folder) => {
                reply.attr(&Duration::new(0, 0), &create_folder_attrs(&folder));
            }
            Err(_err) => match self.service.get_file(ino) {
                Ok(result) => {
                    reply.attr(&Duration::new(0, 0), &create_file_attrs(&result));
                }
                Err(err) => {
                    reply.error(err.into());
                }
            },
        };
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
        reply.error(Error::NotImplemented.into());
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
        log::debug!("access() ino={} mask={}", ino, mask);
        reply.error(libc::ENOSYS);
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
        log::info!(
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
        log::debug!(
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
        log::debug!("bmap() ino={} blocksize={} idx={}", ino, blocksize, idx);
        reply.error(Error::NotImplemented.into());
    }

    fn create(
        &mut self,
        _req: &fuser::Request<'_>,
        parent: u64,
        name: &OsStr,
        mode: u32,
        umask: u32,
        flags: i32,
        reply: fuser::ReplyCreate,
    ) {
        log::debug!(
            "create() parent={} name={:?} mode={} umask={} flags={}",
            parent,
            name,
            mode,
            umask,
            flags
        );
        reply.error(Error::NotImplemented.into());
    }
}
