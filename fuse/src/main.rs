mod config;
mod fs;
mod service;

use clap::Parser;
use fuser::MountOption;
use std::collections::HashMap;
use std::path::PathBuf;

// handle fuse mount arguments like
//
// pcloud-fuse <folder_id> <path_to_mount> -o rw,blabla,dev,suid

fn as_mount_options<'a>(opts: &HashMap<&'a str, Option<&'a str>>) -> Vec<MountOption> {
    let mut result = vec![
        MountOption::AutoUnmount,
        MountOption::NoExec,
        MountOption::NoAtime,
    ];
    if opts.contains_key("ro") {
        result.push(MountOption::RO);
    } else {
        result.push(MountOption::RW);
    }
    result
}

fn init_log_level<'a>(opts: &HashMap<&'a str, Option<&'a str>>) {
    let log_level = match opts.get("log_level") {
        Some(Some(value)) => value.to_string(),
        _ => "info".to_string(),
    };
    tracing_subscriber::fmt()
        .with_env_filter(log_level)
        .try_init()
        .expect("couldn't init logger");
}

fn get_config_path<'a>(opts: &HashMap<&'a str, Option<&'a str>>) -> PathBuf {
    match opts.get("config_path") {
        Some(Some(value)) => PathBuf::from(value),
        _ => PathBuf::from("/etc/pcloud.conf"),
    }
}

#[derive(Parser)]
#[clap(about, author, version)]
struct Opts {
    #[clap(long, short)]
    options: Option<String>,
    // TODO use the folder id for the root path
    folder_id: u64,
    mount_point: PathBuf,
}

impl Opts {
    fn options<'a>(&'a self) -> HashMap<&'a str, Option<&'a str>> {
        self.options
            .as_ref()
            .map(|inner| {
                inner
                    .split(",")
                    .map(|item| {
                        if let Some((key, value)) = item.split_once('=') {
                            (key, Some(value))
                        } else {
                            (item, None)
                        }
                    })
                    .collect::<HashMap<_, _>>()
            })
            .unwrap_or_default()
    }
}

fn main() {
    let opts = Opts::parse();
    let options = opts.options();
    init_log_level(&options);

    let cfg = config::Config::from_path(&get_config_path(&options)).unwrap_or_default();
    let client = cfg.build().expect("couldn't build client");

    let service = service::PCloudService::new(client);

    let mnt_opts = as_mount_options(&options);

    fuser::mount2(
        fs::PCloudFs::new(service),
        opts.mount_point.to_str().unwrap(),
        &mnt_opts,
    )
    .expect("couldn't mount");
}
