use pcloud::binary::BinaryClient;
use pcloud::entry::{Entry, File, Folder};
use pcloud::error::Error as PCloudError;
use std::cell::RefCell;
use std::collections::HashMap;
use std::sync::atomic::{AtomicU64, Ordering};
use std::sync::Mutex;
use std::time::Duration;
use ttl_cache::TtlCache;

pub enum Error {
    BehaviorUndefined,
    InvalidArgument,
    Network,
    NotFound,
    NotImplemented,
    PermissionDenied,
}

impl Error {
    #[tracing::instrument]
    fn from_code(code: u16, message: String) -> Self {
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

impl From<Error> for i32 {
    fn from(err: Error) -> i32 {
        match err {
            Error::BehaviorUndefined => libc::EPERM,
            Error::InvalidArgument => libc::EINVAL,
            Error::Network => libc::EIO,
            Error::NotFound => libc::ENOENT,
            Error::NotImplemented => libc::ENOSYS,
            Error::PermissionDenied => libc::EACCES,
        }
    }
}

enum EntryId {
    File { handle: usize },
    Folder,
}

struct EntryHandle {
    inner: EntryId,
    read: bool,
    write: bool,
}

impl EntryHandle {
    fn folder(read: bool, write: bool) -> Self {
        Self {
            inner: EntryId::Folder,
            read,
            write,
        }
    }

    fn file(handle: usize, read: bool, write: bool) -> Self {
        Self {
            inner: EntryId::File { handle },
            read,
            write,
        }
    }
}

pub struct PCloudService {
    inner: BinaryClient,
    //
    next_handle: AtomicU64,
    entry_handles: Mutex<HashMap<u64, EntryHandle>>,
    //
    file_duration: Duration,
    file_cache: RefCell<TtlCache<u64, File>>,
    folder_duration: Duration,
    folder_cache: RefCell<TtlCache<u64, Folder>>,
}

impl PCloudService {
    pub fn new(inner: BinaryClient) -> Self {
        Self {
            inner,
            //
            next_handle: AtomicU64::new(1),
            entry_handles: Mutex::new(HashMap::new()),
            //
            file_duration: Duration::from_secs(2),
            file_cache: RefCell::new(TtlCache::new(100)),
            folder_duration: Duration::from_secs(5),
            folder_cache: RefCell::new(TtlCache::new(20)),
        }
    }
}

impl Default for PCloudService {
    fn default() -> Self {
        Self::new(BinaryClient::from_env().expect("couldn't build client"))
    }
}

// get file
impl PCloudService {
    fn add_file(&self, inode: u64, file: File) -> File {
        self.file_cache
            .borrow_mut()
            .insert(inode, file.clone(), self.file_duration);
        file
    }

    fn get_file_from_cache(&self, inode: u64) -> Option<File> {
        self.file_cache.borrow_mut().get(&inode).cloned()
    }

    pub fn fetch_file(&mut self, inode: u64) -> Result<File, Error> {
        self.inner
            .get_info_file(inode as usize - 1)
            .map(|res| self.add_file(inode, res.metadata))
            .map_err(|err| {
                tracing::error!("unable to fetch file: {:?}", err);
                Error::Network
            })
    }

    pub fn get_file(&mut self, inode: u64) -> Result<File, Error> {
        if let Some(file) = self.get_file_from_cache(inode) {
            Ok(file)
        } else {
            self.fetch_file(inode)
        }
    }
}

// get folder
impl PCloudService {
    fn add_folder(&self, inode: u64, folder: Folder) -> Folder {
        self.folder_cache
            .borrow_mut()
            .insert(inode, folder.clone(), self.folder_duration);
        folder
    }

    fn get_folder_from_cache(&self, inode: u64) -> Option<Folder> {
        self.folder_cache.borrow().get(&inode).cloned()
    }

    fn remove_folder_from_cache(&mut self, inode: u64) -> Option<Folder> {
        self.folder_cache.borrow_mut().remove(&inode)
    }

    pub fn fetch_folder(&mut self, inode: u64) -> Result<Folder, Error> {
        let params = pcloud::folder::list::Params::new(inode as usize - 1);
        self.inner
            .list_folder(&params)
            .map(|res| self.add_folder(inode, res))
            .map_err(|err| {
                tracing::error!("unable to fetch file: {:?}", err);
                Error::Network
            })
    }

    pub fn get_folder(&mut self, inode: u64) -> Result<Folder, Error> {
        if let Some(folder) = self.get_folder_from_cache(inode) {
            Ok(folder)
        } else {
            self.fetch_folder(inode)
        }
    }
}

impl PCloudService {
    fn allocate_entry(&mut self, entry: EntryHandle) -> u64 {
        let handle = self.next_handle.fetch_add(1, Ordering::SeqCst);
        let mut handles = self
            .entry_handles
            .lock()
            .expect("entry_handles lock is poisoned");
        handles.insert(handle, entry);
        handle
    }

    pub fn can_read(&self, fh: u64) -> bool {
        let handles = self
            .entry_handles
            .lock()
            .expect("entry_handles lock is poisoned");
        if let Some(value) = handles.get(&fh).map(|x| x.read) {
            value
        } else {
            tracing::error!("Undefined entry handle: {}", fh);
            false
        }
    }

    pub fn can_write(&self, fh: u64) -> bool {
        let handles = self
            .entry_handles
            .lock()
            .expect("entry_handles lock is poisoned");
        if let Some(value) = handles.get(&fh).map(|x| x.write) {
            value
        } else {
            tracing::error!("Undefined entry handle: {}", fh);
            false
        }
    }

    pub fn release(&mut self, fh: u64) {
        tracing::trace!("release fh={}", fh);
        let mut handles = self
            .entry_handles
            .lock()
            .expect("entry_handles lock is poisoned");
        handles.remove(&fh);
    }

    fn get_file_handle(&self, fh: u64) -> Option<usize> {
        let handles = self
            .entry_handles
            .lock()
            .expect("entry_handles lock is poisoned");
        handles.get(&fh).and_then(|item| match item.inner {
            EntryId::File { handle, .. } => Some(handle),
            _ => None,
        })
    }
}

fn decode_flag(flags: i32) -> Result<(bool, bool), Error> {
    match flags as i32 & libc::O_ACCMODE {
        libc::O_RDONLY => {
            // Behavior is undefined, but most filesystems return EACCES
            if flags as i32 & libc::O_TRUNC != 0 {
                Err(Error::BehaviorUndefined)
            } else {
                Ok((true, false))
            }
        }
        libc::O_WRONLY => Ok((false, true)),
        libc::O_RDWR => Ok((true, true)),
        // Exactly one access mode flag must be specified
        _ => Err(Error::BehaviorUndefined),
    }
}

// open folder
impl PCloudService {
    fn allocate_folder(&mut self, read: bool, write: bool) -> u64 {
        self.allocate_entry(EntryHandle::folder(read, write))
    }

    pub fn open_folder(&mut self, _inode: u64, flags: i32) -> Result<u64, Error> {
        let (read, write) = decode_flag(flags)?;
        Ok(self.allocate_folder(read, write))
    }
}

// open file
impl PCloudService {
    fn allocate_file(&mut self, handle: usize, read: bool, write: bool) -> u64 {
        self.allocate_entry(EntryHandle::file(handle, read, write))
    }

    pub fn open_file(&mut self, inode: u64, flags: i32) -> Result<u64, Error> {
        let file_id = (inode - 1) as usize;
        let (read, write) = decode_flag(flags)?;
        let params = if write {
            pcloud::fileops::open::Params::new(0x0002).identifier(file_id.into())
        } else {
            pcloud::fileops::open::Params::new(0x0000).identifier(file_id.into())
        };
        let handle = self.inner.file_open(&params).map_err(|err| {
            tracing::error!("unable to fetch file: {:?}", err);
            Error::Network
        })?;
        Ok(self.allocate_file(handle, read, write))
    }
}

// read file
impl PCloudService {
    pub fn read_file(
        &mut self,
        inode: u64,
        fh: u64,
        offset: i64,
        size: u32,
    ) -> Result<Vec<u8>, Error> {
        if !self.can_read(fh) {
            tracing::warn!("does not have right to read on inode={} fh={}", inode, fh);
            return Err(Error::PermissionDenied);
        }
        tracing::trace!("reading len={} from fh={} at offset={}", size, fh, offset);
        let file = self.get_file(inode)?;
        if file.size == Some(0) {
            tracing::warn!("file ino={} fh={} is empty", inode, fh);
            return Ok(Vec::new());
        }
        let handle = self.get_file_handle(fh).ok_or(Error::InvalidArgument)?;
        let params = pcloud::fileops::pread::Params::new(handle, size as usize, offset as usize);
        self.inner
            .file_pread(&params)
            .map(|data| {
                tracing::trace!("received {} bytes", data.len());
                data
            })
            .map_err(|err| {
                tracing::error!("unable to read: {:?}", err);
                Error::Network
            })
    }
}

impl PCloudService {
    pub fn create_file(&mut self, parent: u64, name: &str) -> Result<u64, Error> {
        let params = pcloud::fileops::open::Params::new(0x0042)
            .folder_id((parent - 1) as usize)
            .name(name.to_string());
        let handle = self
            .inner
            .file_open(&params)
            .map(|item| item as u64)
            .map_err(|err| {
                tracing::error!(
                    "unable to read parent={}, name={:?}: {:?}",
                    parent,
                    name,
                    err
                );
                Error::Network
            })?;
        Ok(self.allocate_file(handle as usize, true, true))
    }
}

impl PCloudService {
    pub fn write_file(
        &mut self,
        inode: u64,
        fh: u64,
        offset: i64,
        data: &[u8],
    ) -> Result<usize, Error> {
        if !self.can_write(fh) {
            tracing::warn!("does not have right to write on inode={} fh={}", inode, fh);
            return Err(Error::PermissionDenied);
        }
        tracing::trace!(
            "writing len={} to fh={} at offset={}",
            data.len(),
            fh,
            offset
        );
        let handle = self.get_file_handle(fh).ok_or(Error::InvalidArgument)?;
        let params = pcloud::fileops::pwrite::Params::new(handle, offset as usize, data);
        self.inner.file_pwrite(&params).map_err(|err| {
            tracing::error!("unable to read inode={}, fh={}: {:?}", inode, fh, err);
            Error::Network
        })
    }
}

impl PCloudService {
    pub fn remove_file(&mut self, parent: u64, fname: &str) -> Result<(), Error> {
        let folder = self.get_folder(parent)?;
        if let Some(file) = folder.find_file(fname) {
            self.folder_cache.borrow_mut().remove(&parent);
            self.inner
                .delete_file(file.file_id)
                .map(|_| ())
                .map_err(|err| {
                    tracing::error!("unable to read: {:?}", err);
                    Error::Network
                })
        } else {
            Err(Error::NotFound)
        }
    }
}

impl PCloudService {
    fn rename_file(
        &mut self,
        parent: &Folder,
        file: &File,
        new_parent: &Folder,
        new_name: &str,
    ) -> Result<(), Error> {
        if parent.folder_id != new_parent.folder_id {
            self.remove_folder_from_cache(parent.folder_id as u64 + 1);
            self.remove_folder_from_cache(new_parent.folder_id as u64 + 1);
            let params = pcloud::file::rename::Params::new_move(file.file_id, new_parent.folder_id);
            self.inner.rename_file(&params).map_err(|err| {
                tracing::error!("unable to read: {:?}", err);
                Error::Network
            })?;
        }
        if file.base.name != new_name {
            self.remove_folder_from_cache(parent.folder_id as u64 + 1);
            let params = pcloud::file::rename::Params::new_rename(file.file_id, new_name);
            self.inner.rename_file(&params).map_err(|err| {
                tracing::error!("unable to read: {:?}", err);
                Error::Network
            })?;
        }
        Ok(())
    }

    fn rename_folder(
        &mut self,
        parent: &Folder,
        folder: &Folder,
        new_parent: &Folder,
        new_name: &str,
    ) -> Result<(), Error> {
        if parent.folder_id != new_parent.folder_id {
            self.remove_folder_from_cache(parent.folder_id as u64 + 1);
            self.remove_folder_from_cache(new_parent.folder_id as u64 + 1);
            let params =
                pcloud::folder::rename::Params::new_move(folder.folder_id, new_parent.folder_id);
            self.inner.rename_folder(&params).map_err(|err| {
                tracing::error!("unable to read: {:?}", err);
                Error::Network
            })?;
        }
        if folder.base.name != new_name {
            self.remove_folder_from_cache(parent.folder_id as u64 + 1);
            let params = pcloud::folder::rename::Params::new_rename(folder.folder_id, new_name);
            self.inner.rename_folder(&params).map_err(|err| {
                tracing::error!("unable to read: {:?}", err);
                Error::Network
            })?;
        }
        Ok(())
    }

    pub fn rename(
        &mut self,
        parent_id: u64,
        name: &str,
        new_parent_id: u64,
        new_name: &str,
    ) -> Result<(), Error> {
        let parent = self.get_folder(parent_id)?;
        let entry = parent.find_entry(name).ok_or(Error::NotFound)?;
        let new_parent = self.get_folder(new_parent_id)?;
        match entry {
            Entry::File(file) => self.rename_file(&parent, file, &new_parent, new_name),
            Entry::Folder(folder) => self.rename_folder(&parent, folder, &new_parent, new_name),
        }
    }
}

impl PCloudService {
    pub fn create_folder(&mut self, parent_id: u64, name: &str) -> Result<Folder, Error> {
        self.remove_folder_from_cache(parent_id);
        let params = pcloud::folder::create::Params::new(name, parent_id as usize - 1);
        self.inner.create_folder(&params).map_err(|err| {
            tracing::error!("unable to read: {:?}", err);
            Error::Network
        })
    }
}

impl PCloudService {
    pub fn remove_folder(&mut self, parent_id: u64, name: &str) -> Result<Folder, Error> {
        let parent = self.get_folder(parent_id)?;
        self.remove_folder_from_cache(parent_id);
        let folder = parent.find_folder(name).ok_or(Error::NotFound)?;
        self.inner.delete_folder(folder.folder_id).map_err(|err| {
            tracing::error!("unable to read: {:?}", err);
            Error::Network
        })
    }
}
