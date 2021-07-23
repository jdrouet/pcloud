#[derive(Copy, Clone, Debug)]
pub enum DataCenter {
    Europe,
    UnitedStates,
    #[cfg(test)]
    Test,
}

impl DataCenter {
    pub fn base_url(&self) -> &str {
        match self {
            Self::Europe => "https://eapi.pcloud.com",
            Self::UnitedStates => "https://api.pcloud.com",
            #[cfg(test)]
            #[allow(deprecated)]
            Self::Test => mockito::SERVER_URL,
        }
    }
}
