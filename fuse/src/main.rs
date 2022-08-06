mod config;
mod fs;
mod service;

use clap::Parser;
use fuser::MountOption;
use std::path::PathBuf;

fn parse_mount_option(s: &str) -> MountOption {
    match s {
        "auto_unmount" => MountOption::AutoUnmount,
        "allow_other" => MountOption::AllowOther,
        "allow_root" => MountOption::AllowRoot,
        "default_permissions" => MountOption::DefaultPermissions,
        "dev" => MountOption::Dev,
        "nodev" => MountOption::NoDev,
        "suid" => MountOption::Suid,
        "nosuid" => MountOption::NoSuid,
        "ro" => MountOption::RO,
        "rw" => MountOption::RW,
        "exec" => MountOption::Exec,
        "noexec" => MountOption::NoExec,
        "atime" => MountOption::Atime,
        "noatime" => MountOption::NoAtime,
        "dirsync" => MountOption::DirSync,
        "sync" => MountOption::Sync,
        "async" => MountOption::Async,
        x if x.starts_with("fsname=") => MountOption::FSName(x[7..].into()),
        x if x.starts_with("subtype=") => MountOption::Subtype(x[8..].into()),
        x => MountOption::CUSTOM(x.into()),
    }
}

fn parse_mount_options(input: &str) -> Vec<MountOption> {
    input.split(',').map(parse_mount_option).collect()
}

fn get_mount_option<'a>(opts: &'a [MountOption], expected_key: &'static str) -> Option<&'a str> {
    opts.iter().find_map(|item| match item {
        MountOption::CUSTOM(value) => value.split_once('=').and_then(|(key, value)| {
            if key == expected_key {
                Some(value)
            } else {
                None
            }
        }),
        _ => None,
    })
}

fn get_log_level(opts: &[MountOption]) -> String {
    get_mount_option(opts, "log_level")
        .map(String::from)
        .unwrap_or_else(|| String::from("info"))
}

fn set_log_level(opts: &[MountOption]) {
    let log_level = get_log_level(opts);
    let level = match log_level.as_str() {
        "debug" => "info,pcloud_fuse=debug,fuser=debug",
        _ => "info",
    };
    tracing_subscriber::fmt()
        .with_env_filter(level)
        .try_init()
        .expect("couldn't init logger");
}

fn get_config_path(opts: &[MountOption]) -> PathBuf {
    get_mount_option(opts, "config_path")
        .map(PathBuf::from)
        .unwrap_or_else(|| PathBuf::from("/etc/pcloud.json"))
}

fn filter_mount_options(opts: Vec<MountOption>) -> Vec<MountOption> {
    opts.into_iter()
        .filter(|item| !matches!(item, MountOption::CUSTOM(_)))
        .collect()
}

#[derive(Parser)]
#[clap(about, author, version)]
struct Opts {
    #[clap(short)]
    options: Option<String>,
    mount_point: String,
}

impl Opts {
    fn options(&self) -> Vec<MountOption> {
        self.options
            .as_ref()
            .map(|value| parse_mount_options(value.as_str()))
            .unwrap_or_default()
    }
}

fn main() {
    let opts = Opts::parse();

    let options = opts.options();
    set_log_level(&options);
    let cfg_path = get_config_path(&options);

    // remove pcloud related options
    let options = filter_mount_options(options);

    let cfg = config::Config::from_path(&cfg_path).unwrap_or_default();
    let client = cfg.build().expect("couldn't build client");

    // TODO allow to specify the temporary directory
    let root = tempfile::TempDir::new().expect("couldn't create temporary folder");

    let service = service::Service::new(client, root.path().to_path_buf());

    fuser::mount2(
        fs::PCloudFs::new(service),
        opts.mount_point.as_str(),
        &options,
    )
    .expect("couldn't mount");
}
