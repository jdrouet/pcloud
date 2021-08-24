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
    pub fn init() {
        let _ = env_logger::builder().is_test(true).try_init();
    }
}
