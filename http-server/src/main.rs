mod render;
mod router;

use clap::Parser;
use pcloud::Client;
use std::{fmt::Write, net::IpAddr, str::FromStr, string::FromUtf8Error, sync::Arc};
use tower_http::trace::TraceLayer;
use tracing_subscriber::{layer::SubscriberExt, util::SubscriberInitExt};

#[derive(Parser, Debug)]
#[clap(author, version, about, long_about = None)]
pub struct Config {
    // The host to bind the server
    #[clap(long, default_value = "127.0.0.1", env = "HOST")]
    host: IpAddr,
    // The port to bind the server
    #[clap(long, default_value = "8080", env = "PORT")]
    port: u16,
    // The root folder
    #[clap(long, default_value = "/", env = "ROOT_FOLDER")]
    root_folder: String,
}

impl Config {
    pub fn binding(&self) -> std::net::SocketAddr {
        std::net::SocketAddr::from((self.host, self.port))
    }
}

#[derive(Debug, Clone)]
struct CloudPath {
    inner: Vec<String>,
}

impl FromStr for CloudPath {
    type Err = FromUtf8Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        let mut inner = Vec::new();
        for section in input.split('/').filter(|item| !item.is_empty()) {
            let decoded = urlencoding::decode(section)?;
            inner.push(decoded.into_owned());
        }
        Ok(CloudPath { inner })
    }
}

impl CloudPath {
    fn is_root(&self) -> bool {
        self.inner.is_empty()
    }

    pub fn encoded(&self) -> EncodedCloudPath<'_> {
        EncodedCloudPath(self)
    }

    pub fn raw(&self) -> RawCloudPath<'_> {
        RawCloudPath(self)
    }
}

pub struct RawCloudPath<'a>(&'a CloudPath);

impl std::fmt::Display for RawCloudPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_root() {
            f.write_str("/")
        } else {
            for item in self.0.inner.iter() {
                f.write_char('/')?;
                f.write_str(item.as_ref())?;
            }
            Ok(())
        }
    }
}

struct EncodedCloudPath<'a>(&'a CloudPath);

impl std::fmt::Display for EncodedCloudPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.is_root() {
            f.write_str("/")
        } else {
            for item in self.0.inner.iter() {
                f.write_char('/')?;
                urlencoding::encode(item.as_ref()).fmt(f)?;
            }
            Ok(())
        }
    }
}

#[derive(Debug, Clone)]
struct FolderCloudPath {
    inner: CloudPath,
}

impl FromStr for FolderCloudPath {
    type Err = FromUtf8Error;

    fn from_str(input: &str) -> Result<Self, Self::Err> {
        CloudPath::from_str(input).map(|inner| Self { inner })
    }
}

struct RawFolderCloudPath<'a>(&'a FolderCloudPath);

impl std::fmt::Display for RawFolderCloudPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.inner.is_root() {
            f.write_str("/")
        } else {
            write!(f, "{}/", self.0.inner.raw())
        }
    }
}

struct EncodedFolderCloudPath<'a>(&'a FolderCloudPath);

impl std::fmt::Display for EncodedFolderCloudPath<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        if self.0.inner.is_root() {
            f.write_str("/")
        } else {
            write!(f, "{}/", self.0.inner.encoded())
        }
    }
}

impl FolderCloudPath {
    pub fn with_encoded_file(&self, filename: &str) -> String {
        format!("{}{}", self.encoded(), urlencoding::encode(filename))
    }

    pub fn with_encoded_folder(&self, name: &str) -> String {
        format!("{}{}/", self.encoded(), urlencoding::encode(name))
    }

    pub fn join_folder(&self, other: FolderCloudPath) -> Self {
        let mut inner = self.inner.inner.clone();
        inner.extend(other.inner.inner);
        FolderCloudPath {
            inner: CloudPath { inner },
        }
    }

    pub fn join_file(&self, other: CloudPath) -> CloudPath {
        let mut inner = self.inner.inner.clone();
        inner.extend(other.inner);
        CloudPath { inner }
    }

    pub fn into_inner(self) -> CloudPath {
        self.inner
    }

    pub fn raw(&self) -> RawFolderCloudPath {
        RawFolderCloudPath(self)
    }

    pub fn encoded(&self) -> EncodedFolderCloudPath {
        EncodedFolderCloudPath(self)
    }
}

#[derive(Clone)]
struct Storage(Arc<Client>);

impl Storage {
    fn new(client: Client) -> Self {
        Self(Arc::new(client))
    }
}

impl AsRef<Client> for Storage {
    fn as_ref(&self) -> &Client {
        &self.0
    }
}

#[derive(Debug)]
struct InnerRootPrefix {
    path: FolderCloudPath,
}

#[derive(Clone, Debug)]
struct RootPrefix(Arc<InnerRootPrefix>);

impl RootPrefix {
    fn new(path: &str) -> Self {
        RootPrefix(Arc::new(InnerRootPrefix {
            path: FolderCloudPath::from_str(path).unwrap(),
        }))
    }

    fn root_path(&self) -> &FolderCloudPath {
        &self.0.path
    }
}

#[tokio::main]
async fn main() -> std::io::Result<()> {
    tracing_subscriber::registry()
        .with(
            tracing_subscriber::EnvFilter::try_from_default_env().unwrap_or_else(|_| {
                // axum logs rejections from built-in extractors with the `axum::rejection`
                // target, at `TRACE` level. `axum::rejection=trace` enables showing those events
                "pcloud_http_server=debug,pcloud=debug,tower_http=debug,axum::rejection=trace"
                    .into()
            }),
        )
        .with(tracing_subscriber::fmt::layer())
        .init();

    let args = Config::parse();
    let client = pcloud::builder::ClientBuilder::from_env()
        .build()
        .expect("couldn't build client");

    let storage = Storage::new(client);
    let root_prefix = RootPrefix::new(args.root_folder.as_str());

    let app = router::router()
        .layer(TraceLayer::new_for_http())
        .layer(axum::Extension(storage))
        .layer(axum::Extension(root_prefix));

    let binding = args.binding();
    tracing::info!("serving on {binding}");
    let listener = tokio::net::TcpListener::bind(binding).await.unwrap();
    axum::serve(listener, app).await
}
