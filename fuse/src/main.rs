mod config;
mod fs;
mod service;

use clap::Parser;
use fuser::MountOption;
use std::path::PathBuf;

#[derive(Parser)]
#[clap(about, author, version)]
struct Opts {
    /// Path to load the configuration file. Default to ~/.config/pcloud.json. If not found, loading from environment.
    #[clap(short, long)]
    config: Option<PathBuf>,
    #[clap(long, default_value = "info")]
    log_level: String,
    // #[clap(long, short)]
    // read_only: bool,
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
        let level = match self.log_level.as_str() {
            "debug" => "info,pcloud_fuse=debug,fuser=debug",
            _ => "info",
        };
        tracing_subscriber::fmt()
            .with_env_filter(level)
            .try_init()
            .expect("couldn't init logger");
    }

    fn options(&self) -> Vec<MountOption> {
        vec![
            MountOption::AutoUnmount,
            MountOption::NoExec,
            MountOption::NoAtime,
            MountOption::RO,
        ]
    }
}

fn main() {
    let opts = Opts::parse();
    opts.set_log_level();

    let cfg = config::Config::from_path(&opts.config()).unwrap_or_default();
    let client = cfg.build().expect("couldn't build client");

    // TODO allow to specify the temporary directory
    let root = tempfile::TempDir::new().expect("couldn't create temporary folder");

    let service = service::Service::new(client, root.path().to_path_buf());

    let options = opts.options();

    fuser::mount2(
        fs::PCloudFs::new(service),
        opts.mount_point.to_str().unwrap(),
        &options,
    )
    .expect("couldn't mount");
}
