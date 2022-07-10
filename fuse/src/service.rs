use pcloud::entry::{Entry, File, Folder};
use pcloud::error::Error as PCloudError;
use pcloud::file::checksum::FileCheckSumCommand;
use pcloud::file::download::FileDownloadCommand;
use pcloud::folder::list::FolderListCommand;
use pcloud::http::HttpClient;
use pcloud::prelude::HttpCommand;
use std::collections::{HashMap, HashSet};
use std::fs;
use std::io::prelude::*;
use std::io::SeekFrom;
use std::path::{Path, PathBuf};
use std::time::{Duration, UNIX_EPOCH};
use tokio::runtime::Runtime;
use ttl_cache::TtlCache;

#[inline]
fn inode_to_pcloud_id(id: u64) -> u64 {
    id - 1
}

#[inline]
fn pcloud_id_to_inode(id: u64) -> u64 {
    id + 1
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

pub struct FolderEntry {
    pub id: u64,
    pub position: i64,
    pub file_type: fuser::FileType,
    pub name: String,
}

impl FolderEntry {
    pub fn new(entry: Entry, position: usize) -> Self {
        match entry {
            Entry::File(file) => Self {
                id: pcloud_id_to_inode(file.file_id),
                position: position as i64 + 1,
                file_type: fuser::FileType::RegularFile,
                name: file.base.name,
            },
            Entry::Folder(folder) => Self {
                id: pcloud_id_to_inode(folder.folder_id),
                position: position as i64 + 1,
                file_type: fuser::FileType::Directory,
                name: folder.base.name,
            },
        }
    }
}

pub enum Error {
    BehaviorUndefined,
    InvalidArgument,
    Network,
    NotFound,
    NotImplemented,
    PermissionDenied,
}

impl Error {
    fn from_code(code: u16, message: String) -> Self {
        tracing::debug!("received error {}: {:?}", code, message);
        Self::InvalidArgument
    }
}

impl From<PCloudError> for Error {
    fn from(err: PCloudError) -> Self {
        match err {
            PCloudError::Protocol(code, message) => Self::from_code(code, message),
            _ => Self::Network,
        }
    }
}

impl Error {
    pub fn into_code(self) -> i32 {
        match self {
            Self::BehaviorUndefined => libc::EPERM,
            Self::InvalidArgument => libc::EINVAL,
            Self::Network => libc::EIO,
            Self::NotFound => libc::ENOENT,
            Self::NotImplemented => libc::ENOSYS,
            Self::PermissionDenied => libc::EACCES,
        }
    }
}

pub struct Service {
    runtime: Runtime,
    cache_root: PathBuf,

    client: HttpClient,
    handler_count: u64,

    dir_cache_duration: Duration,
    file_cache_duration: Duration,

    // inode => Folder
    dir_cache: TtlCache<u64, Folder>,
    // handler => inode
    dir_handlers: HashMap<u64, u64>,
    // inode => handler
    dir_inodes: HashMap<u64, HashSet<u64>>,

    // inode => file
    file_cache: TtlCache<u64, File>,
    // inode => (handler, file path)
    file_inodes: HashMap<u64, (HashSet<u64>, PathBuf)>,
    // handler => file
    file_handlers: HashMap<u64, fs::File>,
}

impl Service {
    pub fn new(client: HttpClient, cache_root: PathBuf) -> Self {
        Self {
            runtime: Runtime::new().unwrap(),
            cache_root,
            client,
            handler_count: 0,
            dir_cache: TtlCache::new(20),
            dir_cache_duration: Duration::from_secs(5),
            dir_handlers: Default::default(),
            dir_inodes: Default::default(),
            file_cache: TtlCache::new(100),
            file_cache_duration: Duration::from_secs(5),
            file_handlers: Default::default(),
            file_inodes: Default::default(),
        }
    }

    fn next_handler(&mut self) -> u64 {
        self.handler_count += 1;
        self.handler_count
    }
}

impl Service {
    fn fetch_folder(&mut self, inode: u64) -> Result<Folder, Error> {
        Ok(self.runtime.block_on(async {
            FolderListCommand::new(inode_to_pcloud_id(inode).into())
                .execute(&self.client)
                .await
        })?)
    }

    fn get_folder(&mut self, inode: u64, with_children: bool) -> Result<Folder, Error> {
        if let Some(found) = self.dir_cache.get(&inode) {
            if !with_children || found.contents.is_some() {
                return Ok(found.clone());
            }
        }
        let found = self.fetch_folder(inode)?;
        self.dir_cache
            .insert(inode, found.clone(), self.dir_cache_duration.clone());
        if let Some(ref children) = found.contents {
            for entry in children {
                match entry {
                    Entry::File(file) => {
                        self.file_cache.insert(
                            pcloud_id_to_inode(file.file_id),
                            file.clone(),
                            self.file_cache_duration.clone(),
                        );
                    }
                    Entry::Folder(folder) => {
                        self.dir_cache.insert(
                            pcloud_id_to_inode(folder.folder_id),
                            folder.clone(),
                            self.dir_cache_duration.clone(),
                        );
                    }
                }
            }
        }
        Ok(found)
    }

    fn fetch_file(&mut self, inode: u64) -> Result<File, Error> {
        Ok(self.runtime.block_on(async {
            FileCheckSumCommand::new(inode_to_pcloud_id(inode).into())
                .execute(&self.client)
                .await
                .map(|res| res.metadata)
        })?)
    }

    fn get_file(&mut self, inode: u64) -> Result<File, Error> {
        if let Some(found) = self.file_cache.get(&inode) {
            return Ok(found.clone());
        }
        let found = self.fetch_file(inode)?;
        self.file_cache
            .insert(inode, found.clone(), self.file_cache_duration.clone());
        Ok(found)
    }

    fn fetch_entry(&mut self, inode: u64) -> Result<Entry, Error> {
        Ok(self.runtime.block_on(async {
            let folder =
                FolderListCommand::new(inode_to_pcloud_id(inode).into()).execute(&self.client);
            let file =
                FileCheckSumCommand::new(inode_to_pcloud_id(inode).into()).execute(&self.client);
            match tokio::join!(folder, file) {
                (Ok(folder), _) => Ok(Entry::Folder(folder)),
                (_, Ok(cs)) => Ok(Entry::File(cs.metadata)),
                (Err(err), _) => Err(err.into()),
                _ => Err(Error::NotFound),
            }
        })?)
    }

    fn get_entry(&mut self, inode: u64) -> Result<Entry, Error> {
        if let Some(found) = self.dir_cache.get(&inode) {
            return Ok(Entry::Folder(found.clone()));
        }
        if let Some(found) = self.file_cache.get(&inode) {
            return Ok(Entry::File(found.clone()));
        }
        let entry = self.fetch_entry(inode)?;
        match entry {
            Entry::File(ref file) => {
                self.file_cache
                    .insert(inode, file.clone(), self.file_cache_duration.clone());
            }
            Entry::Folder(ref folder) => {
                self.dir_cache
                    .insert(inode, folder.clone(), self.dir_cache_duration.clone());
            }
        };
        Ok(entry)
    }

    pub fn open_directory(&mut self, inode: u64, _flags: i32) -> Result<u64, Error> {
        let folder = self.get_folder(inode, true)?;
        let handler = self.next_handler();
        self.dir_inodes
            .entry(inode)
            .or_insert(HashSet::default())
            .insert(handler);
        self.dir_handlers.insert(handler, inode);
        self.dir_cache
            .insert(inode, folder, self.dir_cache_duration.clone());
        Ok(handler)
    }

    pub fn close_directory(&mut self, inode: u64, handler: u64) -> Result<(), Error> {
        if self.dir_handlers.remove_entry(&handler).is_none() {
            tracing::error!("unable to close directory with handler {}", handler);
            return Err(Error::NotFound);
        }
        if let Some(handlers) = self.dir_inodes.get_mut(&inode) {
            handlers.remove(&handler);
            if handlers.is_empty() {
                tracing::debug!("no more handler open for inode {}", inode);
                self.dir_inodes.remove(&inode);
            }
            Ok(())
        } else {
            tracing::debug!("unable to find open inode {}", inode);
            Err(Error::NotFound)
        }
    }

    pub fn get_attributes(&mut self, inode: u64) -> Result<fuser::FileAttr, Error> {
        self.get_entry(inode)
            .map(|entry| create_entry_attrs(&entry))
    }

    pub fn read_directory(&mut self, inode: u64) -> Result<Vec<FolderEntry>, Error> {
        let folder = self.get_folder(inode, true)?;
        let children = folder.contents.unwrap_or_default();
        Ok(children
            .into_iter()
            .enumerate()
            .map(|(index, entry)| FolderEntry::new(entry, index))
            .collect())
    }

    pub fn read_file_in_directory(
        &mut self,
        inode: u64,
        name: &str,
    ) -> Result<fuser::FileAttr, Error> {
        let folder = self.get_folder(inode, true)?;
        let children = folder.contents.unwrap_or_default();
        children
            .iter()
            .find(|item| item.base().name == name)
            .map(create_entry_attrs)
            .ok_or(Error::NotFound)
    }

    fn download_file(&mut self, inode: u64) -> Result<PathBuf, Error> {
        let path = self.cache_root.join(inode.to_string());
        self.runtime.block_on(async {
            let file = fs::File::create(&path).map_err(|err| {
                tracing::error!("unable to create file localy: {:?}", err);
                Error::PermissionDenied
            })?;
            FileDownloadCommand::new(inode_to_pcloud_id(inode).into(), file)
                .execute(&self.client)
                .await
                .map_err(Error::from)
        })?;
        Ok(path)
    }

    pub fn open_file(&mut self, inode: u64) -> Result<u64, Error> {
        let handler = self.next_handler();
        if let Some((handlers, _)) = self.file_inodes.get_mut(&inode) {
            handlers.insert(handler);
        } else {
            let file_path = self.download_file(inode)?;

            let file = fs::File::open(&file_path).map_err(|err| {
                tracing::error!("unable to open file: {:?}", err);
                Error::BehaviorUndefined
            })?;
            self.file_handlers.insert(handler, file);
            let mut handlers = HashSet::default();
            handlers.insert(handler);
            self.file_inodes.insert(inode, (handlers, file_path));
        }
        Ok(handler)
    }

    pub fn close_file(&mut self, inode: u64, handler: u64, _flush: bool) -> Result<(), Error> {
        self.file_handlers.remove(&handler).ok_or_else(|| {
            tracing::error!("handler not found {}", handler);
            Error::NotFound
        })?;
        let is_empty = match self.file_inodes.get_mut(&inode) {
            Some((handlers, _)) => {
                if !handlers.remove(&handler) {
                    tracing::error!("unable to find handler {} for inode {}", handler, inode);
                    return Err(Error::NotFound);
                }
                handlers.is_empty()
            }
            None => {
                tracing::error!("inode not found {}", inode);
                return Err(Error::NotFound);
            }
        };
        if is_empty {
            let (_, ref file_path) = self.file_inodes.remove(&inode).ok_or_else(|| {
                tracing::error!("inode not found {}", inode);
                Error::NotFound
            })?;
            if let Err(err) = fs::remove_file(file_path) {
                tracing::warn!("unable to remove local file {:?}: {:?}", file_path, err);
            }
        }
        Ok(())
    }

    pub fn read_file(
        &mut self,
        _inode: u64,
        handler: u64,
        offset: i64,
        size: u32,
    ) -> Result<Vec<u8>, Error> {
        if let Some(file) = self.file_handlers.get_mut(&handler) {
            file.seek(SeekFrom::Start(offset as u64)).map_err(|err| {
                tracing::warn!("unable to move in file: {:?}", err);
                Error::PermissionDenied
            })?;
            let mut res = vec![0; size as usize];
            let length = file.read(&mut res).map_err(|err| {
                tracing::warn!("unable to copy file content: {:?}", err);
                Error::BehaviorUndefined
            })?;
            res.resize(length, 0);
            Ok(res)
        } else {
            Err(Error::NotFound)
        }
    }
}
