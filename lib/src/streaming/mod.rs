pub mod get_audio_link;
pub mod get_file_link;
pub mod get_video_link;

#[derive(Debug, serde::Deserialize)]
pub struct SteamingLinkList {
    // expires: String,
    pub hosts: Vec<String>,
    pub path: String,
}

#[cfg(feature = "client-http")]
pub struct StreamingLink<'a> {
    host: &'a str,
    path: &'a str,
}

#[cfg(feature = "client-http")]
impl<'a> StreamingLink<'a> {
    #[inline(always)]
    fn new(host: &'a str, path: &'a str) -> Self {
        Self { host, path }
    }
}

#[cfg(feature = "client-http")]
impl<'a> std::fmt::Display for StreamingLink<'a> {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        write!(f, "https://{}{}", self.host, self.path)
    }
}

#[cfg(feature = "client-http")]
impl SteamingLinkList {
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
