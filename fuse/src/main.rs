mod config;
mod fs;
mod service;

use clap::{crate_authors, crate_description, crate_version, Clap};
use fuser::MountOption;
use pcloud::binary::BinaryClient;
use std::path::PathBuf;

#[derive(Clap)]
#[clap(about = crate_description!(), author = crate_authors!(), version = crate_version!())]
struct Opts {
    #[clap(
        short,
        long,
        about = "Path to load the configuration file. Default to ~/.config/pcloud.json. If not found, loading from environment."
    )]
    config: Option<PathBuf>,
    #[clap(long, default_value = "info")]
    log_level: String,
    mount_point: PathBuf,
}

impl Opts {
    fn config(&self) -> PathBuf {
        if let Some(ref cfg) = self.config {
            cfg.clone()
        } else if let Some(cfg_dir) = dirs::config_dir() {
            cfg_dir.join("pcloud.json")
        } else {
            PathBuf::from(".pcloud.json")
        }
    }

    fn set_log_level(&self) {
        tracing_subscriber::fmt()
            .with_env_filter(self.log_level.clone())
            .try_init()
            .expect("couldn't init logger");
    }
}

fn main() {
    let opts = Opts::parse();
    opts.set_log_level();

    let client = if let Ok(config) = config::Config::from_path(&opts.config()) {
        config.build()
    } else {
        BinaryClient::from_env()
    }
    .expect("couldn't create client");

    let service = service::PCloudService::new(client);

    let options = vec![
        MountOption::AutoUnmount,
        MountOption::NoExec,
        MountOption::NoAtime,
    ];

    fuser::mount2(
        fs::PCloudFs::new(service),
        opts.mount_point.to_str().unwrap(),
        &options,
    )
    .expect("couldn't mount");
}
