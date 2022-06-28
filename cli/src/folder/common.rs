use sha2::{Digest, Sha256};
use std::path::Path;

pub(crate) fn get_checksum(path: &Path) -> Result<String, String> {
    let mut file = std::fs::File::open(path)
        .map_err(|err| format!("unable to open file {:?}: {:?}", path, err))?;
    let mut hasher = Sha256::new();
    std::io::copy(&mut file, &mut hasher)
        .map_err(|err| format!("unable to compute hash for {:?}: {:?}", path, err))?;
    Ok(hex::encode(hasher.finalize()))
}
