#[derive(Clone, Debug)]
pub struct Region {
    http_url: String,
    binary_url: String,
}

impl Region {
    pub fn new(http_url: String, binary_url: String) -> Self {
        Self {
            http_url,
            binary_url,
        }
    }

    pub fn eu() -> Self {
        Self::new(
            "https://eapi.pcloud.com".into(),
            "eapi.pcloud.com:8398".into(),
        )
    }

    pub fn us() -> Self {
        Self::new(
            "https://api.pcloud.com".into(),
            "api.pcloud.com:8398".into(),
        )
    }

    #[cfg(test)]
    pub fn mock() -> Self {
        Self::new(
            mockito::server_url(),
            format!("{}:{}", mockito::server_url(), 8398),
        )
    }
}

impl Region {
    pub fn http_url(&self) -> &str {
        self.http_url.as_str()
    }

    pub fn binary_url(&self) -> &str {
        self.binary_url.as_str()
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
        let binary_url = std::env::var("PCLOUD_REGION_BINARY_URL").ok()?;
        log::debug!(
            "creating region from split env {}, {}",
            http_url,
            binary_url
        );

        Some(Self::new(http_url, binary_url))
    }

    fn from_name_env() -> Option<Self> {
        let name = std::env::var("PCLOUD_REGION").ok()?;
        log::debug!("creating region from name {}", name);
        match name.as_str() {
            "eu" => Some(Self::eu()),
            "us" => Some(Self::us()),
            _ => None,
        }
    }

    pub fn from_env() -> Self {
        Self::from_split_env()
            .or_else(Self::from_name_env)
            .unwrap_or_else(Self::default)
    }
}
