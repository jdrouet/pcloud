use pcloud::binary::BinaryClient;
use pcloud::entry::{File, Folder};
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
    File { file_id: usize, handle: usize },
    Folder { folder_id: usize },
}

impl EntryId {
    fn inner(&self) -> usize {
        match self {
            Self::File { file_id, .. } => *file_id,
            Self::Folder { folder_id } => *folder_id,
        }
    }
}

struct EntryHandle {
    inner: EntryId,
    read: bool,
    write: bool,
}

impl EntryHandle {
    fn folder(folder_id: usize, read: bool, write: bool) -> Self {
        Self {
            inner: EntryId::Folder { folder_id },
            read,
            write,
        }
    }

    fn file(file_id: usize, handle: usize, read: bool, write: bool) -> Self {
        Self {
            inner: EntryId::File { file_id, handle },
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

    fn fetch_file(&mut self, inode: u64) -> Result<File, Error> {
        self.inner
            .get_info_file(inode as usize - 1)
            .map(|res| res.metadata)
            .map_err(|err| {
                log::warn!("unable to fetch file: {:?}", err);
                Error::Network
            })
    }

    pub fn get_file(&mut self, inode: u64) -> Result<File, Error> {
        if let Some(file) = self.get_file_from_cache(inode) {
            Ok(file)
        } else {
            self.fetch_file(inode).map(|res| self.add_file(inode, res))
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

    fn fetch_folder(&mut self, inode: u64) -> Result<Folder, Error> {
        let params = pcloud::folder::list::Params::new(inode as usize - 1);
        self.inner.list_folder(&params).map_err(|err| {
            log::warn!("unable to fetch folder: {:?}", err);
            Error::Network
        })
    }

    pub fn get_folder(&mut self, inode: u64) -> Result<Folder, Error> {
        if let Some(folder) = self.get_folder_from_cache(inode) {
            Ok(folder)
        } else {
            self.fetch_folder(inode)
                .map(|res| self.add_folder(inode, res))
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
            log::error!("Undefined entry handle: {}", fh);
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
            log::error!("Undefined entry handle: {}", fh);
            false
        }
    }

    pub fn release(&mut self, fh: u64) {
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
    fn allocate_folder(&mut self, folder_id: usize, read: bool, write: bool) -> u64 {
        self.allocate_entry(EntryHandle::folder(folder_id, read, write))
    }

    pub fn open_folder(&mut self, inode: u64, flags: i32) -> Result<u64, Error> {
        let folder_id = (inode - 1) as usize;
        let (read, write) = decode_flag(flags)?;
        Ok(self.allocate_folder(folder_id, read, write))
    }
}

fn build_flag(_read: bool, write: bool) -> u16 {
    if write {
        0x0042
    } else {
        0x0000
    }
}

// open file
impl PCloudService {
    fn allocate_file(&mut self, file_id: usize, handle: usize, read: bool, write: bool) -> u64 {
        self.allocate_entry(EntryHandle::file(file_id, handle, read, write))
    }

    pub fn open_file(&mut self, inode: u64, flags: i32) -> Result<u64, Error> {
        let file_id = (inode - 1) as usize;
        let (read, write) = decode_flag(flags)?;
        let params =
            pcloud::fileops::open::Params::new(build_flag(read, write)).identifier(file_id.into());
        let handle = self.inner.file_open(&params).map_err(|err| {
            log::warn!("unable to open file: {:?}", err);
            Error::Network
        })?;
        Ok(self.allocate_file(file_id, handle, read, write))
    }
}

// read file
impl PCloudService {
    pub fn read_file(
        &mut self,
        _inode: u64,
        fh: u64,
        offset: i64,
        size: u32,
    ) -> Result<Vec<u8>, Error> {
        if !self.can_read(fh) {
            return Err(Error::PermissionDenied);
        }
        let handle = self.get_file_handle(fh).ok_or(Error::InvalidArgument)?;
        let params = pcloud::fileops::pread::Params::new(handle, size as usize, offset as usize);
        self.inner.file_pread(&params).map_err(|err| {
            log::warn!("unable to read file: {:?}", err);
            Error::Network
        })
    }
}
