mod auth;
pub mod binary;
pub mod credentials;
mod date;
pub mod entry;
pub mod error;
pub mod file;
pub mod fileops;
pub mod folder;
pub mod http;
pub mod region;
pub mod request;

#[cfg(test)]
mod tests {
    use rand::distributions::Alphanumeric;
    use rand::Rng;

    pub fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }

    pub fn random_name() -> String {
        rand::thread_rng()
            .sample_iter(&Alphanumeric)
            .take(16)
            .map(char::from)
            .collect()
    }
}
