use std::io::Read;
use std::path::PathBuf;

pub(crate) fn compute_sha1(path: &PathBuf) -> std::io::Result<String> {
    let mut file = std::fs::OpenOptions::new().read(true).open(path)?;
    let mut buffer = [0u8; 4096];
    let mut hasher = sha1_smol::Sha1::new();
    loop {
        let size = file.read(&mut buffer)?;
        if size == 0 {
            break;
        }
        hasher.update(&buffer[..size]);
    }
    Ok(hasher.digest().to_string())
}
