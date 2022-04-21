pub mod prelude;

pub mod auth;
pub mod binary;
pub mod credentials;
mod date;
pub mod entry;
pub mod error;
pub mod file;
pub mod fileops;
pub mod folder;
pub mod general;
pub mod http;
pub mod region;
pub mod request;
pub mod streaming;

#[cfg(test)]
mod tests {
    use rand::distributions::Alphanumeric;
    use rand::Rng;

    pub fn init() {
        let _ = tracing_subscriber::fmt()
            .with_env_filter("debug")
            .try_init();
    }

    pub fn random_name() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect()
    }
}
