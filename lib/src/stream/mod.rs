use chrono::{DateTime, Utc};

pub mod audio;
pub mod file;
pub mod video;

/// A struct that represents a list of streaming links with metadata such as expiration date and hosts.
#[derive(Debug, serde::Deserialize)]
pub struct StreamingLinkList {
    /// The expiration date and time of the streaming links.
    #[serde(with = "crate::date")]
    pub expires: DateTime<Utc>,

    /// The list of available host URLs for streaming.
    pub hosts: Vec<String>,

    /// The path to the resource to be streamed.
    pub path: String,
}

/// A struct representing an individual streaming link, built using a host and a path.
pub struct StreamingLink<'a> {
    host: &'a str,
    path: &'a str,
}

impl<'a> StreamingLink<'a> {
    /// Creates a new `StreamingLink` from a host and path.
    ///
    /// # Arguments
    ///
    /// * `host` - The host URL for streaming.
    /// * `path` - The path to the resource.
    ///
    /// # Returns
    ///
    /// Returns a new `StreamingLink`.
    #[inline(always)]
    fn new(host: &'a str, path: &'a str) -> Self {
        Self { host, path }
    }
}

impl std::fmt::Display for StreamingLink<'_> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "https://{}{}", self.host, self.path)
    }
}

impl StreamingLinkList {
    /// Returns the first streaming link using the first host in the list.
    ///
    /// # Returns
    ///
    /// An optional `StreamingLink`, which may be `None` if the list of hosts is empty.
    pub fn first_link(&self) -> Option<StreamingLink<'_>> {
        self.hosts
            .first()
            .map(|host| StreamingLink::new(host.as_str(), self.path.as_str()))
    }

    /// Returns the last streaming link using the last host in the list.
    ///
    /// # Returns
    ///
    /// An optional `StreamingLink`, which may be `None` if the list of hosts is empty.
    pub fn last_link(&self) -> Option<StreamingLink<'_>> {
        self.hosts
            .last()
            .map(|host| StreamingLink::new(host.as_str(), self.path.as_str()))
    }

    /// Returns an iterator over all available streaming links.
    ///
    /// This will create a `StreamingLink` for each host in the list using the common path.
    ///
    /// # Returns
    ///
    /// An iterator over `StreamingLink` instances.
    pub fn links(&self) -> impl Iterator<Item = StreamingLink<'_>> {
        self.hosts
            .iter()
            .map(move |host| StreamingLink::new(host.as_str(), self.path.as_str()))
    }
}
