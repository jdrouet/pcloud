use pcloud::entry::{File, Folder};
use sha2::{Digest, Sha256};
use std::path::Path;
use std::str::FromStr;

/// Checks that a folder contains a file based on its filename
pub(crate) fn contains_file<'f>(folder: &'f Folder, fname: &str) -> Option<&'f File> {
    folder
        .contents
        .as_ref()
        .and_then(|contents| contents.iter().find(|item| item.base().name == fname))
        .and_then(|entry| entry.as_file())
}

/// Computes the sha256 checksum of a local file
pub(crate) fn get_checksum(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path)
        .map_err(|err| format!("unable to open file {:?}: {:?}", path, err))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)
        .map_err(|err| format!("unable to compute hash for {:?}: {:?}", path, err))?;
    Ok(hex::encode(hasher.finalize()))
}

/// Method to compare existing files
pub(crate) enum CompareMethod {
    /// Compute the checksum of the existing file and compares it with the remote file
    Checksum,
    /// Force upload event if the file already exists
    Force,
    /// Just checks the presence of the file, do not compare anything
    Presence,
}

impl FromStr for CompareMethod {
    type Err = String;
    fn from_str(value: &str) -> Result<Self, Self::Err> {
        match value {
            "checksum" => Ok(Self::Checksum),
            "force" => Ok(Self::Force),
            "presence" => Ok(Self::Presence),
            _ => Err(format!("invalid comparison method {:?}", value)),
        }
    }
}
