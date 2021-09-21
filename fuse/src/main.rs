mod fs;
mod service;

use clap::{AppSettings, Clap};
use fuser::MountOption;

#[derive(Clap)]
#[clap(setting = AppSettings::ColoredHelp)]
struct Opts {
    #[clap(long, default_value = "0")]
    folder_id: usize,
    #[clap(long, default_value = "/tmp/pcloud-fuse")]
    data_dir: String,
    mount_point: String,
}

fn main() {
    tracing_subscriber::fmt()
        .with_env_filter(std::env::var("LOG").unwrap_or_else(|_| String::from("info")))
        .try_init()
        .expect("couldn't init logger");

    let opts = Opts::parse();

    let service = service::PCloudService::default();

    let options = vec![
        MountOption::AutoUnmount,
        MountOption::NoExec,
        MountOption::NoAtime,
    ];

    fuser::mount2(
        fs::PCloudFs::new(service, opts.folder_id, opts.data_dir),
        opts.mount_point,
        &options,
    )
    .expect("couldn't mount");
}
