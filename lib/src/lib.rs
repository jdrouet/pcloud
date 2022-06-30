pub mod prelude;

pub mod auth;
#[cfg(feature = "client-binary")]
pub mod binary;
pub mod credentials;
mod date;
pub mod entry;
pub mod error;
pub mod file;
#[cfg(feature = "client-binary")]
pub mod fileops;
pub mod folder;
pub mod general;
#[cfg(feature = "client-http")]
pub mod http;
pub mod region;
pub mod request;
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
