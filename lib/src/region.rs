//! The region related module needed for the authentication

/// A representation of a region
#[derive(Clone, Debug)]
pub struct Region {
    http_url: String,
}

impl Region {
    /// Creates a new region with the given parameters.
    ///
    /// This method should be used for testing when mocking the calls to the PCloud servers.
    pub fn new(http_url: String) -> Self {
        Self { http_url }
    }

    /// Creates a region object representing the EU region
    pub fn eu() -> Self {
        Self::new("https://eapi.pcloud.com".into())
    }

    /// Creates a region object representing the US region
    pub fn us() -> Self {
        Self::new("https://api.pcloud.com".into())
    }
}

impl Region {
    pub fn http_url(&self) -> &str {
        self.http_url.as_str()
    }
}

impl Default for Region {
    fn default() -> Self {
        Self::eu()
    }
}

impl Region {
    fn from_split_env() -> Option<Self> {
        let http_url = std::env::var("PCLOUD_REGION_HTTP_URL").ok()?;

        Some(Self::new(http_url))
    }

    /// Creates a region based on the region provided as a `&str`.
    pub fn from_name(name: &str) -> Option<Self> {
        match name {
            "eu" | "EU" => Some(Self::eu()),
            "us" | "US" => Some(Self::us()),
            _ => None,
        }
    }

    fn from_name_env() -> Option<Self> {
        let name = std::env::var("PCLOUD_REGION").ok()?;
        Self::from_name(name.as_str())
    }

    /// Creates a region based on the `PCLOUD_REGION` environment variable value.
    pub fn from_env() -> Option<Self> {
        Self::from_split_env().or_else(Self::from_name_env)
    }
}
