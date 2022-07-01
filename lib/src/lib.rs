/// Utilities for parsing dates
mod date;
/// The traits for implementing the commands
pub mod prelude;
/// The module to handle the responses from the server
pub mod request;

#[cfg(feature = "client-binary")]
pub mod binary;
#[cfg(feature = "client-http")]
pub mod http;

pub mod credentials;
pub mod region;

pub mod entry;
/// The errors thrown by the commands
pub mod error;

/// The [file commands](https://docs.pcloud.com/methods/file/) from the PCloud documentation
pub mod file;
/// The [fileops commands](https://docs.pcloud.com/methods/fileops/) from the PCloud documentation
#[cfg(feature = "client-binary")]
pub mod fileops;
/// The [folder commands](https://docs.pcloud.com/methods/folder/) from the PCloud documentation
pub mod folder;
/// The [general commands](https://docs.pcloud.com/methods/general/) from the PCloud documentation
pub mod general;
/// The [streaming commands](https://docs.pcloud.com/methods/streaming/) from the PCloud documentation
pub mod streaming;

#[cfg(test)]
mod tests {
    use rand::distributions::Alphanumeric;
    use rand::Rng;

    #[allow(dead_code)]
    pub fn init() {
        let filter = std::env::var("RUST_LOG").unwrap_or_else(|_| "info".into());
        let _ = tracing_subscriber::fmt().with_env_filter(filter).try_init();
    }

    #[allow(dead_code)]
    pub fn random_name() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect()
    }
}
