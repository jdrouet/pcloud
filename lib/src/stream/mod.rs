use chrono::{DateTime, Utc};

pub mod audio;
pub mod file;
pub mod video;

#[derive(Debug, serde::Deserialize)]
pub struct StreamingLinkList {
    #[serde(with = "crate::date")]
    pub expires: DateTime<Utc>,
    pub hosts: Vec<String>,
    pub path: String,
}

pub struct StreamingLink<'a> {
    host: &'a str,
    path: &'a str,
}

impl<'a> StreamingLink<'a> {
    #[inline(always)]
    fn new(host: &'a str, path: &'a str) -> Self {
        Self { host, path }
    }
}

impl<'a> std::fmt::Display for StreamingLink<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "https://{}{}", self.host, self.path)
    }
}

impl StreamingLinkList {
    pub fn first_link(&self) -> Option<StreamingLink<'_>> {
        self.hosts
            .first()
            .map(|host| StreamingLink::new(host.as_str(), self.path.as_str()))
    }

    pub fn last_link(&self) -> Option<StreamingLink<'_>> {
        self.hosts
            .last()
            .map(|host| StreamingLink::new(host.as_str(), self.path.as_str()))
    }

    pub fn links(&self) -> impl Iterator<Item = StreamingLink<'_>> {
        self.hosts
            .iter()
            .map(move |host| StreamingLink::new(host.as_str(), self.path.as_str()))
    }
}
